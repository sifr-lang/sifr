// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct __SifrIoNativeFileHandle {
    pub _id: String,
}
impl __SifrIoNativeFileHandle {
    pub fn new(id: String) -> Self {
        let __sifr_field_init_0: String = id;
        Self { _id: __sifr_field_init_0 }
    }
}
impl __SifrIoNativeFileHandle {}
impl ::std::fmt::Display for __SifrIoNativeFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "NativeFileHandle(_id={})", self._id)
    }
}

mod __sifr_project_nominals {
    use crate::__SifrIoNativeFileHandle;
    pub use ::sifr_runtime::SifrInt;
    pub fn datetime_now() -> String {
        ::sifr_stdlib::time::datetime_now()
    }
    pub fn datetime_now_struct() -> Vec<SifrInt> {
        ::sifr_stdlib::time::datetime_now_struct()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn datetime_format(dt: &String, fmt: &String) -> String {
        ::sifr_stdlib::time::datetime_format(dt, fmt)
    }
    pub fn datetime_from_timestamp(ts: f64) -> Result<String, ValueError> {
        ::sifr_stdlib::time::datetime_from_timestamp(ts)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
        __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
            FloatPrecisionLossError,
        ),
    }
    impl From<FloatOverflowError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        fn from(value: FloatOverflowError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                value,
            )
        }
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
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
        __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
            FloatPrecisionLossError,
        ),
        __SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(ValueError),
    }
    impl From<FloatOverflowError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        fn from(value: FloatOverflowError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                value,
            )
        }
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
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub _offset: SifrInt,
    }
    impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub fn new(offset: SifrInt) -> Self {
            let __sifr_field_init_0: SifrInt = offset.clone();
            Self {
                _offset: __sifr_field_init_0,
            }
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub fn offset(&self) -> SifrInt {
            self._offset.clone()
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub fn iso_suffix(&self) -> String {
            let mut sign: String = "+".to_string();
            if (&self._offset.clone() < &SifrInt::from_i64(0)) {
                sign = "-".to_string();
            }
            let mut abs_offset: SifrInt = self._offset.clone();
            if &abs_offset < &SifrInt::from_i64(0) {
                abs_offset = -&abs_offset;
            }
            let h: SifrInt = abs_offset.floor_div_known_nonzero(&SifrInt::from_i64(3600));
            let m: SifrInt = abs_offset
                .floor_mod_known_nonzero(&SifrInt::from_i64(3600))
                .floor_div_known_nonzero(&SifrInt::from_i64(60));
            let mut hs: String = format!("{}", h);
            if (&SifrInt::from(hs.chars().count()) < &SifrInt::from_i64(2)) {
                hs = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + hs.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((hs).as_str());
                    __sifr_concat
                };
            }
            let mut ms: String = format!("{}", m);
            if (&SifrInt::from(ms.chars().count()) < &SifrInt::from_i64(2)) {
                ms = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + ms.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((ms).as_str());
                    __sifr_concat
                };
            }
            {
                let mut __sifr_concat: String = String::with_capacity(
                    ((sign.len() + hs.len()) + 1usize) + ms.len(),
                );
                __sifr_concat.push_str((sign).as_str());
                __sifr_concat.push_str((hs).as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str((ms).as_str());
                __sifr_concat
            }
        }
    }
    impl PartialEq for __SifrStdlib_sifr_x2edatetime_x2etimezone {
        fn eq(&self, other: &__SifrStdlib_sifr_x2edatetime_x2etimezone) -> bool {
            self._offset == other._offset
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2etimezone {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            if (&self._offset.clone() == &SifrInt::from_i64(0)) {
                return write!(f, "{}", "UTC".to_string());
            }
            write!(
                f, "{}", { let mut __sifr_concat : String = String::with_capacity(3usize +
                0usize); __sifr_concat.push_str("UTC"); __sifr_concat.push_str((self
                .iso_suffix()).as_str()); __sifr_concat }
            )
        }
    }
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub year: SifrInt,
        pub month: SifrInt,
        pub day: SifrInt,
        pub hour: SifrInt,
        pub minute: SifrInt,
        pub second: SifrInt,
        pub microsecond: SifrInt,
        pub _tz_offset: Option<SifrInt>,
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn new(
            year: SifrInt,
            month: SifrInt,
            day: SifrInt,
            hour: SifrInt,
            minute: SifrInt,
            second: SifrInt,
            microsecond: SifrInt,
            tz_offset: Option<SifrInt>,
        ) -> Self {
            let __sifr_field_init_0: SifrInt = year.clone();
            let __sifr_field_init_1: SifrInt = month.clone();
            let __sifr_field_init_2: SifrInt = day.clone();
            let __sifr_field_init_3: SifrInt = hour.clone();
            let __sifr_field_init_4: SifrInt = minute.clone();
            let __sifr_field_init_5: SifrInt = second.clone();
            let __sifr_field_init_6: SifrInt = microsecond.clone();
            let __sifr_field_init_7: Option<SifrInt> = tz_offset.clone();
            Self {
                year: __sifr_field_init_0,
                month: __sifr_field_init_1,
                day: __sifr_field_init_2,
                hour: __sifr_field_init_3,
                minute: __sifr_field_init_4,
                second: __sifr_field_init_5,
                microsecond: __sifr_field_init_6,
                _tz_offset: __sifr_field_init_7,
            }
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn isoformat(&self) -> String {
            let y: String = format!("{}", self.year.clone());
            let mut mo: String = format!("{}", self.month.clone());
            if (&SifrInt::from(mo.chars().count()) < &SifrInt::from_i64(2)) {
                mo = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mo).as_str());
                    __sifr_concat
                };
            }
            let mut d: String = format!("{}", self.day.clone());
            if (&SifrInt::from(d.chars().count()) < &SifrInt::from_i64(2)) {
                d = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((d).as_str());
                    __sifr_concat
                };
            }
            let mut h: String = format!("{}", self.hour.clone());
            if (&SifrInt::from(h.chars().count()) < &SifrInt::from_i64(2)) {
                h = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((h).as_str());
                    __sifr_concat
                };
            }
            let mut mi: String = format!("{}", self.minute.clone());
            if (&SifrInt::from(mi.chars().count()) < &SifrInt::from_i64(2)) {
                mi = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mi).as_str());
                    __sifr_concat
                };
            }
            let mut s: String = format!("{}", self.second.clone());
            if (&SifrInt::from(s.chars().count()) < &SifrInt::from_i64(2)) {
                s = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + s.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((s).as_str());
                    __sifr_concat
                };
            }
            let mut base: String = {
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
            };
            if (&self.microsecond.clone() != &SifrInt::from_i64(0)) {
                base.push('.');
                base.push_str((_six_digits(self.microsecond.clone())).as_str());
            }
            let tz_offset_opt: Option<SifrInt> = self._tz_offset.clone();
            if let Some(tz_offset_opt) = tz_offset_opt.clone() {
                let offset: SifrInt = tz_offset_opt.clone();
                let mut sign: String = "+".to_string();
                let mut abs_offset: SifrInt = offset.clone();
                if &abs_offset < &SifrInt::from_i64(0) {
                    sign = "-".to_string();
                    abs_offset = -&abs_offset;
                }
                let h_off: SifrInt = abs_offset
                    .floor_div_known_nonzero(&SifrInt::from_i64(3600));
                let m_off: SifrInt = abs_offset
                    .floor_mod_known_nonzero(&SifrInt::from_i64(3600))
                    .floor_div_known_nonzero(&SifrInt::from_i64(60));
                let mut hs_off: String = format!("{}", h_off);
                if (&SifrInt::from(hs_off.chars().count()) < &SifrInt::from_i64(2)) {
                    hs_off = {
                        let mut __sifr_concat: String = String::with_capacity(
                            1usize + hs_off.len(),
                        );
                        __sifr_concat.push('0');
                        __sifr_concat.push_str((hs_off).as_str());
                        __sifr_concat
                    };
                }
                let mut ms_off: String = format!("{}", m_off);
                if (&SifrInt::from(ms_off.chars().count()) < &SifrInt::from_i64(2)) {
                    ms_off = {
                        let mut __sifr_concat: String = String::with_capacity(
                            1usize + ms_off.len(),
                        );
                        __sifr_concat.push('0');
                        __sifr_concat.push_str((ms_off).as_str());
                        __sifr_concat
                    };
                }
                return {
                    let mut __sifr_concat: String = String::with_capacity(
                        (((base.len() + sign.len()) + hs_off.len()) + 1usize) + ms_off.len(),
                    );
                    __sifr_concat.push_str((base).as_str());
                    __sifr_concat.push_str((sign).as_str());
                    __sifr_concat.push_str((hs_off).as_str());
                    __sifr_concat.push(':');
                    __sifr_concat.push_str((ms_off).as_str());
                    __sifr_concat
                };
            }
            base
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn timestamp(&self) -> SifrInt {
            let mut days: SifrInt = SifrInt::from_i64(0);
            if (&self.year.clone() >= &SifrInt::from_i64(1970)) {
                let mut y: SifrInt = SifrInt::from_i64(1970);
                while (&y < &self.year.clone()) {
                    days = &days + &_days_in_year((y).clone());
                    y = &y + &SifrInt::from_i64(1);
                }
            } else {
                let mut y: SifrInt = SifrInt::from_i64(1969);
                while (&y >= &self.year.clone()) {
                    days = &days - &_days_in_year((y).clone());
                    y = &y - &SifrInt::from_i64(1);
                }
            }
            let mut m: SifrInt = SifrInt::from_i64(1);
            while (&m < &self.month.clone()) {
                days = &days + &_days_in_month(self.year.clone(), (m).clone());
                m = &m + &SifrInt::from_i64(1);
            }
            days = &(&days + &self.day.clone()) - &SifrInt::from_i64(1);
            let naive_timestamp: SifrInt = &(&(&(&days * &SifrInt::from_i64(86400))
                + &(&self.hour.clone() * &SifrInt::from_i64(3600)))
                + &(&self.minute.clone() * &SifrInt::from_i64(60))) + &self.second.clone();
            let tz_offset_opt: Option<SifrInt> = self._tz_offset.clone();
            if let Some(tz_offset_opt) = tz_offset_opt.clone() {
                let offset: SifrInt = tz_offset_opt.clone();
                return &naive_timestamp - &offset;
            }
            naive_timestamp
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn timestamp_microseconds(&self) -> SifrInt {
            &(&self.timestamp() * &SifrInt::from_i64(1000000)) + &self.microsecond.clone()
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn astimezone(
            &self,
            tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
        ) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
            let mut target: __SifrStdlib_sifr_x2edatetime_x2etimezone = __SifrStdlib_sifr_x2edatetime_x2etimezone::new(
                SifrInt::from_i64(0),
            );
            if let Some(tz) = tz.as_ref() {
                let __sifr_try_res: Result<(), ValueError> = (|| {
                    let tz_text: String = format!("{}", tz);
                    let target_offset: SifrInt = _timezone_offset_from_text(&tz_text)?;
                    target = __SifrStdlib_sifr_x2edatetime_x2etimezone::new(
                        (target_offset).clone(),
                    );
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
            }
            _from_timestamp_microseconds_with_tz(
                self.timestamp_microseconds(),
                &Some((target).clone()),
            )
        }
    }
    impl PartialEq for __SifrStdlib_sifr_x2edatetime_x2edatetime {
        fn eq(&self, other: &__SifrStdlib_sifr_x2edatetime_x2edatetime) -> bool {
            let same_tz: bool = self._tz_offset == other._tz_offset;
            (((((((((self.year.clone() == other.year.clone()))
                && ((self.month.clone() == other.month.clone())))
                && ((self.day.clone() == other.day.clone())))
                && ((self.hour.clone() == other.hour.clone())))
                && ((self.minute.clone() == other.minute.clone())))
                && ((self.second.clone() == other.second.clone())))
                && ((self.microsecond.clone() == other.microsecond.clone()))) && (same_tz))
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2edatetime {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    pub fn _is_leap_year(year: SifrInt) -> bool {
        (((&year.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == &SifrInt::from_i64(0))
            && (&year.floor_mod_known_nonzero(&SifrInt::from_i64(100))
                != &SifrInt::from_i64(0)))
            || ((&year.floor_mod_known_nonzero(&SifrInt::from_i64(400))
                == &SifrInt::from_i64(0))))
    }
    pub fn _days_in_year(year: SifrInt) -> SifrInt {
        if _is_leap_year((year).clone()) {
            return SifrInt::from_i64(366);
        }
        SifrInt::from_i64(365)
    }
    pub fn _days_in_month(year: SifrInt, month: SifrInt) -> SifrInt {
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
    pub fn _substring(value: &String, start: SifrInt, end: SifrInt) -> String {
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
    pub fn _six_digits(value: SifrInt) -> String {
        let mut rendered: String = format!("{}", value);
        let mut __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
        while (&SifrInt::from(__sifr_chars_rendered.len()) < &SifrInt::from_i64(6)) {
            rendered = {
                let mut __sifr_concat: String = String::with_capacity(
                    1usize + rendered.len(),
                );
                __sifr_concat.push('0');
                __sifr_concat.push_str((rendered).as_str());
                __sifr_concat
            };
            __sifr_chars_rendered = rendered.chars().collect::<Vec<char>>();
        }
        rendered
    }
    pub fn _parse_datetime_iso(
        value: &String,
    ) -> Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError> {
        let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        if (&SifrInt::from(__sifr_chars_value.len()) < &SifrInt::from_i64(19)) {
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
        if ((((({
            let __indexed_char_option = __sifr_chars_value
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(4))))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        }) != "-")
            || (({
                let __indexed_char_option = __sifr_chars_value
                    .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(7))))
                    .map(|c| c.to_string());
                __indexed_char_option.as_slice()[0_usize].clone()
            }) != "-"))
            || (({
                let __indexed_char_option = __sifr_chars_value
                    .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(10))))
                    .map(|c| c.to_string());
                __indexed_char_option.as_slice()[0_usize].clone()
            }) != "T"))
            || (({
                let __indexed_char_option = __sifr_chars_value
                    .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(13))))
                    .map(|c| c.to_string());
                __indexed_char_option.as_slice()[0_usize].clone()
            }) != ":"))
            || (({
                let __indexed_char_option = __sifr_chars_value
                    .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(16))))
                    .map(|c| c.to_string());
                __indexed_char_option.as_slice()[0_usize].clone()
            }) != ":")
        {
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
        let __sifr_try_res: Result<
            Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError>,
            ParseError,
        > = (|| {
            let year: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(0), SifrInt::from_i64(4))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let month: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(5), SifrInt::from_i64(7))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let day: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(8), SifrInt::from_i64(10))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let hour: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(11), SifrInt::from_i64(13))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let minute: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(14), SifrInt::from_i64(16))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let second: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(17), SifrInt::from_i64(19))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            Ok(
                Ok((
                    year.clone(),
                    month.clone(),
                    day.clone(),
                    hour.clone(),
                    minute.clone(),
                    second.clone(),
                )),
            )
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let _e = __sifr_try_err.clone();
                return Err(ValueError::new("invalid datetime string".to_string()));
            }
        }
    }
    pub fn _timezone_offset_from_text(text: &String) -> Result<SifrInt, ValueError> {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if (text).as_str() == "UTC" {
            return Ok(SifrInt::from_i64(0));
        }
        if (&SifrInt::from(__sifr_chars_text.len()) != &SifrInt::from_i64(9)) {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        if (_substring(text, SifrInt::from_i64(0), SifrInt::from_i64(3)) != "UTC") {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        let sign_value: String = _substring(
            text,
            SifrInt::from_i64(3),
            SifrInt::from_i64(4),
        );
        if (sign_value != "+") && (sign_value != "-") {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        if (__sifr_chars_text
            .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(6))))
            .map(|c| c.to_string()) != Some(":".to_string()))
        {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        let __sifr_try_res: Result<Result<SifrInt, ValueError>, ParseError> = (|| {
            let hours: SifrInt = SifrInt::parse_decimal(
                    &(_substring(text, SifrInt::from_i64(4), SifrInt::from_i64(6))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let minutes: SifrInt = SifrInt::parse_decimal(
                    &(_substring(text, SifrInt::from_i64(7), SifrInt::from_i64(9))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let mut offset: SifrInt = &(&hours * &SifrInt::from_i64(3600))
                + &(&minutes * &SifrInt::from_i64(60));
            if sign_value == "-" {
                offset = -&offset;
            }
            Ok(Ok(offset))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let _e = __sifr_try_err.clone();
                return Err(ValueError::new("invalid timezone string".to_string()));
            }
        }
    }
    pub fn _from_timestamp_with_tz(
        ts: f64,
        tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
    ) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
        let __sifr_try_res: Result<
            Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
        > = (|| {
            let whole_seconds: SifrInt = (SifrInt::from_f64_trunc(ts)
                .ok_or_else(|| ValueError {
                    message: "cannot convert non-finite float to int".to_string(),
                }))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            let whole_seconds_float: f64 = (whole_seconds
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
                .map_err(|__e| match __e {
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
                })?;
            let fractional: f64 = ts - whole_seconds_float;
            let mut microsecond: SifrInt = (SifrInt::from_f64_trunc(
                    fractional * (1000000.0_f64),
                )
                .ok_or_else(|| ValueError {
                    message: "cannot convert non-finite float to int".to_string(),
                }))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            if &microsecond < &SifrInt::from_i64(0) {
                microsecond = -&microsecond;
            }
            let mut adjusted_seconds: SifrInt = whole_seconds.clone();
            let mut tz_offset_value: SifrInt = SifrInt::from_i64(0);
            let mut tz_has_offset: bool = false;
            if let Some(tz) = tz.as_ref() {
                let tz_text: String = format!("{}", tz);
                let tz_offset: SifrInt = (_timezone_offset_from_text(&tz_text))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                        __e,
                    ))?;
                adjusted_seconds = &whole_seconds + &tz_offset;
                tz_offset_value = tz_offset;
                tz_has_offset = true;
            }
            let adjusted_seconds_float: f64 = (adjusted_seconds
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
                .map_err(|__e| match __e {
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
                })?;
            let rendered: String = (datetime_from_timestamp(adjusted_seconds_float))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            let parts: (SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt) = (_parse_datetime_iso(
                &rendered,
            ))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            let year_part: Option<SifrInt> = Some((parts).0.clone());
            let month_part: Option<SifrInt> = Some((parts).1.clone());
            let day_part: Option<SifrInt> = Some((parts).2.clone());
            let hour_part: Option<SifrInt> = Some((parts).3.clone());
            let minute_part: Option<SifrInt> = Some((parts).4.clone());
            let second_part: Option<SifrInt> = Some((parts).5.clone());
            let mut year: SifrInt = SifrInt::from_i64(0);
            let mut month: SifrInt = SifrInt::from_i64(1);
            let mut day: SifrInt = SifrInt::from_i64(1);
            let mut hour: SifrInt = SifrInt::from_i64(0);
            let mut minute: SifrInt = SifrInt::from_i64(0);
            let mut second: SifrInt = SifrInt::from_i64(0);
            if let Some(year_part) = year_part.clone() {
                year = year_part;
            }
            if let Some(month_part) = month_part.clone() {
                month = month_part;
            }
            if let Some(day_part) = day_part.clone() {
                day = day_part;
            }
            if let Some(hour_part) = hour_part.clone() {
                hour = hour_part;
            }
            if let Some(minute_part) = minute_part.clone() {
                minute = minute_part;
            }
            if let Some(second_part) = second_part.clone() {
                second = second_part;
            }
            if tz_has_offset {
                return Ok(
                    Ok(
                        __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                            (year).clone(),
                            (month).clone(),
                            (day).clone(),
                            (hour).clone(),
                            (minute).clone(),
                            (second).clone(),
                            (microsecond).clone(),
                            Some(tz_offset_value),
                        ),
                    ),
                );
            }
            Ok(
                Ok(
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        (year).clone(),
                        (month).clone(),
                        (day).clone(),
                        (hour).clone(),
                        (minute).clone(),
                        (second).clone(),
                        (microsecond).clone(),
                        None,
                    ),
                ),
            )
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                match __sifr_try_err {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                }
            }
        }
    }
    pub fn _from_timestamp_microseconds_with_tz(
        value: SifrInt,
        tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
    ) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
        let whole_seconds: SifrInt = value
            .floor_div_known_nonzero(&SifrInt::from_i64(1000000));
        let microsecond: SifrInt = value
            .floor_mod_known_nonzero(&SifrInt::from_i64(1000000));
        let __sifr_try_res: Result<
            Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
        > = (|| {
            let whole_seconds_float: f64 = (whole_seconds
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
                .map_err(|__e| match __e {
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
                })?;
            let result: __SifrStdlib_sifr_x2edatetime_x2edatetime = (_from_timestamp_with_tz(
                whole_seconds_float,
                tz,
            ))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            Ok(
                Ok(
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        result.year.clone(),
                        result.month.clone(),
                        result.day.clone(),
                        result.hour.clone(),
                        result.minute.clone(),
                        result.second.clone(),
                        (microsecond).clone(),
                        result._tz_offset.clone(),
                    ),
                ),
            )
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                match __sifr_try_err {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                }
            }
        }
    }
    pub fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
        ::sifr_stdlib::random::random_int(
                ::sifr_runtime::interop::SifrIntBridge::from(min),
                ::sifr_runtime::interop::SifrIntBridge::from(max),
            )
            .into_sifr_int()
    }
    pub fn random_float() -> f64 {
        ::sifr_stdlib::random::random_float()
    }
    pub fn random_word_to_unit_float(value: SifrInt) -> f64 {
        ::sifr_stdlib::random::random_word_to_unit_float(
            ::sifr_runtime::interop::SifrIntBridge::from(value),
        )
    }
    pub fn random_seed() -> SifrInt {
        ::sifr_stdlib::random::random_seed().into_sifr_int()
    }
    pub fn random_uniform(min: f64, max: f64) -> f64 {
        ::sifr_stdlib::random::random_uniform(min, max)
    }
    pub fn random_randrange(
        start: SifrInt,
        stop: SifrInt,
        step: SifrInt,
    ) -> Result<SifrInt, ValueError> {
        ::sifr_stdlib::random::random_randrange(
                ::sifr_runtime::interop::SifrIntBridge::from(start),
                ::sifr_runtime::interop::SifrIntBridge::from(stop),
                ::sifr_runtime::interop::SifrIntBridge::from(step),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn random_gauss(mu: f64, sigma: f64) -> f64 {
        ::sifr_stdlib::random::random_gauss(mu, sigma)
    }
    pub fn random_module_state_words() -> Vec<SifrInt> {
        ::sifr_stdlib::random::random_module_state_words()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn random_module_state_index() -> SifrInt {
        ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
    }
    pub fn random_module_state_gauss_next() -> Option<f64> {
        ::sifr_stdlib::random::random_module_state_gauss_next()
    }
    pub fn random_module_set_state(
        words: &Vec<SifrInt>,
        index: SifrInt,
        gauss_next: Option<f64>,
    ) -> Result<(), ValueError> {
        ::sifr_stdlib::random::random_module_set_state(
                &words
                    .iter()
                    .cloned()
                    .map(::sifr_runtime::interop::SifrIntBridge::from)
                    .collect::<Vec<_>>(),
                ::sifr_runtime::interop::SifrIntBridge::from(index),
                gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn base64_encode(s: &String) -> String {
        ::sifr_stdlib::base64::base64_encode(s)
    }
    pub fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::base64::base64_encode_bytes(data)
    }
    pub fn base64_decode(s: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::base64_decode(s)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
        ::sifr_stdlib::base64::base64_decode_bytes(data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn base64_encode_opts(
        s: &String,
        altchars: &String,
        wrapcol: SifrInt,
    ) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::base64_encode_opts(
                s,
                altchars,
                ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn base64_decode_opts(
        s: &String,
        altchars: &String,
        validate: bool,
        ignorechars: &String,
    ) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn urlsafe_b64encode(s: &String) -> String {
        ::sifr_stdlib::base64::urlsafe_b64encode(s)
    }
    pub fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
    }
    pub fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::urlsafe_b64decode(s)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
        ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn b32encode(s: &String) -> String {
        ::sifr_stdlib::base64::b32encode(s)
    }
    pub fn b32decode(s: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::b32decode(s)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn b32hexencode(s: &String) -> String {
        ::sifr_stdlib::base64::b32hexencode(s)
    }
    pub fn b32hexdecode(s: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::b32hexdecode(s)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha256_bytes(data)
    }
    pub fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::md5_bytes(data)
    }
    pub fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha1_bytes(data)
    }
    pub fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha224_bytes(data)
    }
    pub fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha384_bytes(data)
    }
    pub fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha512_bytes(data)
    }
    pub fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2b_bytes(data)
    }
    pub fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2s_bytes(data)
    }
    pub fn read_text(path: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn write_text(path: &String, content: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn exists(path: &String) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub fn read_lines(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::read_lines(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn append_text(path: &String, content: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::append_text(path, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _open_file(path: &String, mode: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::open_file(path, mode)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_read(handle: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::file_read(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write(handle: &String, data: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_readline(handle: &String) -> Result<Option<String>, IOError> {
        ::sifr_stdlib::fs::file_readline(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_readlines(handle: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::file_readlines(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_close(handle: &String) {
        ::sifr_stdlib::fs::file_close(handle);
    }
    pub fn _file_read_bytes(handle: &String) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::fs::file_read_bytes(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write_bytes(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn open_file(
        path: &String,
        mode: &String,
    ) -> Result<__SifrIoNativeFileHandle, IOError> {
        let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
            let handle_id: String = _open_file(path, mode)?;
            Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message.clone()));
            }
        }
    }
    pub fn file_read(handle: &__SifrIoNativeFileHandle) -> Result<String, IOError> {
        _file_read(&handle._id.clone())
    }
    pub fn file_write(
        handle: &__SifrIoNativeFileHandle,
        data: &String,
    ) -> Result<(), IOError> {
        _file_write(&handle._id.clone(), data)
    }
    pub fn file_readline(
        handle: &__SifrIoNativeFileHandle,
    ) -> Result<Option<String>, IOError> {
        _file_readline(&handle._id.clone())
    }
    pub fn file_readlines(
        handle: &__SifrIoNativeFileHandle,
    ) -> Result<Vec<String>, IOError> {
        _file_readlines(&handle._id.clone())
    }
    pub fn file_close(handle: &__SifrIoNativeFileHandle) {
        _file_close(&handle._id.clone());
    }
    pub fn file_read_bytes(handle: &__SifrIoNativeFileHandle) -> Result<Vec<u8>, IOError> {
        _file_read_bytes(&handle._id.clone())
    }
    pub fn file_write_bytes(
        handle: &__SifrIoNativeFileHandle,
        data: &Vec<u8>,
    ) -> Result<(), IOError> {
        _file_write_bytes(&handle._id.clone(), data)
    }
    pub fn getcwd() -> Result<String, IOError> {
        ::sifr_stdlib::fs::getcwd()
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn listdir(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::listdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn mkdir(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::mkdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rmdir(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn remove_file(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rename(src: &String, dst: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rename(src, dst)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn chdir(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::chdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn stat_size(path: &String) -> Result<SifrInt, IOError> {
        ::sifr_stdlib::fs::stat_size(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn disk_usage(path: &String) -> Vec<SifrInt> {
        ::sifr_stdlib::fs::disk_usage(path)
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn is_file(path: &String) -> bool {
        ::sifr_stdlib::fs::is_file(path)
    }
    pub fn is_dir(path: &String) -> bool {
        ::sifr_stdlib::fs::is_dir(path)
    }
    pub fn copy_file(src: &String, dst: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::copy_file(src, dst)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn walk_dir(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::walk_dir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rmdir_all(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir_all(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn gettempdir() -> String {
        ::sifr_stdlib::fs::gettempdir()
    }
    pub fn makedirs(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::makedirs(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn touch(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::touch(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn resolve_path(path: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::resolve_path(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn iterdir(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::iterdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn glob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::glob_pattern(dir, pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rglob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
        pub _algorithm: String,
        pub _data: Vec<u8>,
        pub name: String,
        pub digest_size: SifrInt,
        pub block_size: SifrInt,
    }
    impl __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
        pub fn new(
            algorithm: String,
            data: Vec<u8>,
            name: String,
            digest_size: SifrInt,
            block_size: SifrInt,
        ) -> Self {
            let __sifr_field_init_0: String = algorithm;
            let __sifr_field_init_1: Vec<u8> = data;
            let __sifr_field_init_2: String = name;
            let __sifr_field_init_3: SifrInt = digest_size.clone();
            let __sifr_field_init_4: SifrInt = block_size.clone();
            Self {
                _algorithm: __sifr_field_init_0,
                _data: __sifr_field_init_1,
                name: __sifr_field_init_2,
                digest_size: __sifr_field_init_3,
                block_size: __sifr_field_init_4,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
        pub fn update(&mut self, data: &Vec<u8>) {
            self._data = {
                let mut __v = (self._data.clone()).clone();
                __v.extend((data).iter().cloned());
                __v
            };
        }
    }
    impl __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
        pub fn hexdigest(&self) -> String {
            _hash_hex(&self._algorithm, &self._data)
        }
    }
    impl __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
        pub fn digest(&self) -> Vec<u8> {
            _hash_bytes(&self._algorithm, &self._data)
        }
    }
    pub fn _hash_bytes(algorithm: &String, data: &Vec<u8>) -> Vec<u8> {
        if (algorithm).as_str() == "md5" {
            return md5_bytes(data);
        } else {
            if (algorithm).as_str() == "sha1" {
                return sha1_bytes(data);
            } else {
                if (algorithm).as_str() == "sha224" {
                    return sha224_bytes(data);
                } else {
                    if (algorithm).as_str() == "sha256" {
                        return sha256_bytes(data);
                    } else {
                        if (algorithm).as_str() == "sha384" {
                            return sha384_bytes(data);
                        } else {
                            if (algorithm).as_str() == "sha512" {
                                return sha512_bytes(data);
                            } else {
                                if (algorithm).as_str() == "blake2b" {
                                    return blake2b_bytes(data);
                                } else {
                                    if (algorithm).as_str() == "blake2s" {
                                        return blake2s_bytes(data);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        {
            let __sifr_empty_bytes_literal: Vec<u8> = vec![];
            __sifr_empty_bytes_literal
        }
    }
    pub fn _hash_hex(algorithm: &String, data: &Vec<u8>) -> String {
        {
            let __bytes_receiver = &_hash_bytes(algorithm, data);
            let mut __hex = String::with_capacity(
                __bytes_receiver.len().saturating_mul(2_usize),
            );
            for __byte in __bytes_receiver.iter() {
                __hex.push_str(&format!("{:02x}", * __byte));
            }
            __hex
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl IOError {
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
    pub fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let __sifr_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match __sifr_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => {
                    "PermissionDenied".to_string()
                }
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                    "DirectoryNotEmpty".to_string()
                }
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
    }
    impl ParseError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
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
    pub struct RegexError {
        pub message: String,
        pub detail: String,
    }
    impl RegexError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                detail: String::new(),
            }
        }
    }
    impl ::std::fmt::Display for RegexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for RegexError {}
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
}
pub use __sifr_project_nominals::FloatOverflowError;
pub use __sifr_project_nominals::FloatPrecisionLossError;
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::RegexError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2edatetime;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2etimezone;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ehashlib_x2eHashObject;
use ::sifr_runtime::SifrInt;
fn datetime_now() -> String {
    ::sifr_stdlib::time::datetime_now()
}
fn datetime_now_struct() -> Vec<SifrInt> {
    ::sifr_stdlib::time::datetime_now_struct()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn datetime_format(dt: &String, fmt: &String) -> String {
    ::sifr_stdlib::time::datetime_format(dt, fmt)
}
fn datetime_from_timestamp(ts: f64) -> Result<String, ValueError> {
    ::sifr_stdlib::time::datetime_from_timestamp(ts)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
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
impl From<FloatOverflowError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn from(value: FloatOverflowError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            value,
        )
    }
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
#[derive(Debug, Clone)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
    __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
        FloatPrecisionLossError,
    ),
    __SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(ValueError),
}
impl From<FloatOverflowError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn from(value: FloatOverflowError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            value,
        )
    }
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
fn _six_digits(value: SifrInt) -> String {
    let mut rendered: String = format!("{}", value);
    let mut __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
    while (&SifrInt::from(__sifr_chars_rendered.len()) < &SifrInt::from_i64(6)) {
        rendered = {
            let mut __sifr_concat: String = String::with_capacity(
                1usize + rendered.len(),
            );
            __sifr_concat.push('0');
            __sifr_concat.push_str((rendered).as_str());
            __sifr_concat
        };
        __sifr_chars_rendered = rendered.chars().collect::<Vec<char>>();
    }
    rendered
}
fn _parse_datetime_iso(
    value: &String,
) -> Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError> {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_value.len()) < &SifrInt::from_i64(19)) {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    if ((((({
        let __indexed_char_option = __sifr_chars_value
            .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(4))))
            .map(|c| c.to_string());
        __indexed_char_option.as_slice()[0_usize].clone()
    }) != "-")
        || (({
            let __indexed_char_option = __sifr_chars_value
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(7))))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        }) != "-"))
        || (({
            let __indexed_char_option = __sifr_chars_value
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(10))))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        }) != "T"))
        || (({
            let __indexed_char_option = __sifr_chars_value
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(13))))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        }) != ":"))
        || (({
            let __indexed_char_option = __sifr_chars_value
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(16))))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        }) != ":")
    {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    let __sifr_try_res: Result<
        Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError>,
        ParseError,
    > = (|| {
        let year: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(0), SifrInt::from_i64(4))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let month: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(5), SifrInt::from_i64(7))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let day: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(8), SifrInt::from_i64(10))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let hour: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(11), SifrInt::from_i64(13))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minute: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(14), SifrInt::from_i64(16))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let second: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(17), SifrInt::from_i64(19))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        Ok(
            Ok((
                year.clone(),
                month.clone(),
                day.clone(),
                hour.clone(),
                minute.clone(),
                second.clone(),
            )),
        )
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
    }
}
fn _timezone_offset_from_text(text: &String) -> Result<SifrInt, ValueError> {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (text).as_str() == "UTC" {
        return Ok(SifrInt::from_i64(0));
    }
    if (&SifrInt::from(__sifr_chars_text.len()) != &SifrInt::from_i64(9)) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (_substring(text, SifrInt::from_i64(0), SifrInt::from_i64(3)) != "UTC") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let sign_value: String = _substring(
        text,
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    );
    if (sign_value != "+") && (sign_value != "-") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (__sifr_chars_text
        .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(6))))
        .map(|c| c.to_string()) != Some(":".to_string()))
    {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let __sifr_try_res: Result<Result<SifrInt, ValueError>, ParseError> = (|| {
        let hours: SifrInt = SifrInt::parse_decimal(
                &(_substring(text, SifrInt::from_i64(4), SifrInt::from_i64(6))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minutes: SifrInt = SifrInt::parse_decimal(
                &(_substring(text, SifrInt::from_i64(7), SifrInt::from_i64(9))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let mut offset: SifrInt = &(&hours * &SifrInt::from_i64(3600))
            + &(&minutes * &SifrInt::from_i64(60));
        if sign_value == "-" {
            offset = -&offset;
        }
        Ok(Ok(offset))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
    }
}
fn _from_timestamp_with_tz(
    ts: f64,
    tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let whole_seconds: SifrInt = (SifrInt::from_f64_trunc(ts)
            .ok_or_else(|| ValueError {
                message: "cannot convert non-finite float to int".to_string(),
            }))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        let whole_seconds_float: f64 = (whole_seconds
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
            .map_err(|__e| match __e {
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
            })?;
        let fractional: f64 = ts - whole_seconds_float;
        let mut microsecond: SifrInt = (SifrInt::from_f64_trunc(
                fractional * (1000000.0_f64),
            )
            .ok_or_else(|| ValueError {
                message: "cannot convert non-finite float to int".to_string(),
            }))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        if &microsecond < &SifrInt::from_i64(0) {
            microsecond = -&microsecond;
        }
        let mut adjusted_seconds: SifrInt = whole_seconds.clone();
        let mut tz_offset_value: SifrInt = SifrInt::from_i64(0);
        let mut tz_has_offset: bool = false;
        if let Some(tz) = tz.as_ref() {
            let tz_text: String = format!("{}", tz);
            let tz_offset: SifrInt = (_timezone_offset_from_text(&tz_text))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            adjusted_seconds = &whole_seconds + &tz_offset;
            tz_offset_value = tz_offset;
            tz_has_offset = true;
        }
        let adjusted_seconds_float: f64 = (adjusted_seconds
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
            .map_err(|__e| match __e {
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
            })?;
        let rendered: String = (datetime_from_timestamp(adjusted_seconds_float))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        let parts: (SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt) = (_parse_datetime_iso(
            &rendered,
        ))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        let year_part: Option<SifrInt> = Some((parts).0.clone());
        let month_part: Option<SifrInt> = Some((parts).1.clone());
        let day_part: Option<SifrInt> = Some((parts).2.clone());
        let hour_part: Option<SifrInt> = Some((parts).3.clone());
        let minute_part: Option<SifrInt> = Some((parts).4.clone());
        let second_part: Option<SifrInt> = Some((parts).5.clone());
        let mut year: SifrInt = SifrInt::from_i64(0);
        let mut month: SifrInt = SifrInt::from_i64(1);
        let mut day: SifrInt = SifrInt::from_i64(1);
        let mut hour: SifrInt = SifrInt::from_i64(0);
        let mut minute: SifrInt = SifrInt::from_i64(0);
        let mut second: SifrInt = SifrInt::from_i64(0);
        if let Some(year_part) = year_part.clone() {
            year = year_part;
        }
        if let Some(month_part) = month_part.clone() {
            month = month_part;
        }
        if let Some(day_part) = day_part.clone() {
            day = day_part;
        }
        if let Some(hour_part) = hour_part.clone() {
            hour = hour_part;
        }
        if let Some(minute_part) = minute_part.clone() {
            minute = minute_part;
        }
        if let Some(second_part) = second_part.clone() {
            second = second_part;
        }
        if tz_has_offset {
            return Ok(
                Ok(
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        (year).clone(),
                        (month).clone(),
                        (day).clone(),
                        (hour).clone(),
                        (minute).clone(),
                        (second).clone(),
                        (microsecond).clone(),
                        Some(tz_offset_value),
                    ),
                ),
            );
        }
        Ok(
            Ok(
                __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                    (year).clone(),
                    (month).clone(),
                    (day).clone(),
                    (hour).clone(),
                    (minute).clone(),
                    (second).clone(),
                    (microsecond).clone(),
                    None,
                ),
            ),
        )
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
            }
        }
    }
}
fn _from_timestamp_microseconds_with_tz(
    value: SifrInt,
    tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
    let whole_seconds: SifrInt = value
        .floor_div_known_nonzero(&SifrInt::from_i64(1000000));
    let microsecond: SifrInt = value
        .floor_mod_known_nonzero(&SifrInt::from_i64(1000000));
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let whole_seconds_float: f64 = (whole_seconds
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
            .map_err(|__e| match __e {
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
            })?;
        let result: __SifrStdlib_sifr_x2edatetime_x2edatetime = (_from_timestamp_with_tz(
            whole_seconds_float,
            tz,
        ))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        Ok(
            Ok(
                __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                    result.year.clone(),
                    result.month.clone(),
                    result.day.clone(),
                    result.hour.clone(),
                    result.minute.clone(),
                    result.second.clone(),
                    (microsecond).clone(),
                    result._tz_offset.clone(),
                ),
            ),
        )
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
            }
        }
    }
}
fn from_timestamp(
    ts: f64,
    tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
    _from_timestamp_with_tz(ts, tz)
}
fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_word_to_unit_float(value: SifrInt) -> f64 {
    ::sifr_stdlib::random::random_word_to_unit_float(
        ::sifr_runtime::interop::SifrIntBridge::from(value),
    )
}
fn random_seed() -> SifrInt {
    ::sifr_stdlib::random::random_seed().into_sifr_int()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(
    start: SifrInt,
    stop: SifrInt,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<SifrInt> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn random_module_state_index() -> SifrInt {
    ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<SifrInt>,
    index: SifrInt,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .cloned()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode(s: &String) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode_opts(
    s: &String,
    altchars: &String,
    wrapcol: SifrInt,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_encode_opts(
            s,
            altchars,
            ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_opts(
    s: &String,
    altchars: &String,
    validate: bool,
    ignorechars: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64encode(s: &String) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
}
fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32encode(s: &String) -> String {
    ::sifr_stdlib::base64::b32encode(s)
}
fn b32decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32hexencode(s: &String) -> String {
    ::sifr_stdlib::base64::b32hexencode(s)
}
fn b32hexdecode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32hexdecode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
}
fn read_text(path: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::read_text(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn write_text(path: &String, content: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::write_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn exists(path: &String) -> bool {
    ::sifr_stdlib::fs::exists(path)
}
fn read_lines(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::read_lines(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn append_text(path: &String, content: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::append_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _open_file(path: &String, mode: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::open_file(path, mode)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_read(handle: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::file_read(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write(handle: &String, data: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readline(handle: &String) -> Result<Option<String>, IOError> {
    ::sifr_stdlib::fs::file_readline(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readlines(handle: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::file_readlines(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_close(handle: &String) {
    ::sifr_stdlib::fs::file_close(handle);
}
fn _file_read_bytes(handle: &String) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::fs::file_read_bytes(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write_bytes(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn open_file(path: &String, mode: &String) -> Result<__SifrIoNativeFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
        let handle_id: String = _open_file(path, mode)?;
        Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message.clone()));
        }
    }
}
fn file_read(handle: &__SifrIoNativeFileHandle) -> Result<String, IOError> {
    _file_read(&handle._id.clone())
}
fn file_write(handle: &__SifrIoNativeFileHandle, data: &String) -> Result<(), IOError> {
    _file_write(&handle._id.clone(), data)
}
fn file_readline(handle: &__SifrIoNativeFileHandle) -> Result<Option<String>, IOError> {
    _file_readline(&handle._id.clone())
}
fn file_readlines(handle: &__SifrIoNativeFileHandle) -> Result<Vec<String>, IOError> {
    _file_readlines(&handle._id.clone())
}
fn file_close(handle: &__SifrIoNativeFileHandle) {
    _file_close(&handle._id.clone());
}
fn file_read_bytes(handle: &__SifrIoNativeFileHandle) -> Result<Vec<u8>, IOError> {
    _file_read_bytes(&handle._id.clone())
}
fn file_write_bytes(
    handle: &__SifrIoNativeFileHandle,
    data: &Vec<u8>,
) -> Result<(), IOError> {
    _file_write_bytes(&handle._id.clone(), data)
}
fn getcwd() -> Result<String, IOError> {
    ::sifr_stdlib::fs::getcwd()
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn listdir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::listdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn mkdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::mkdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn remove_file(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::remove_file(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rename(src: &String, dst: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rename(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn chdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::chdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn stat_size(path: &String) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::stat_size(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn disk_usage(path: &String) -> Vec<SifrInt> {
    ::sifr_stdlib::fs::disk_usage(path)
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn is_file(path: &String) -> bool {
    ::sifr_stdlib::fs::is_file(path)
}
fn is_dir(path: &String) -> bool {
    ::sifr_stdlib::fs::is_dir(path)
}
fn copy_file(src: &String, dst: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::copy_file(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn walk_dir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::walk_dir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir_all(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir_all(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn gettempdir() -> String {
    ::sifr_stdlib::fs::gettempdir()
}
fn makedirs(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::makedirs(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn touch(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::touch(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn resolve_path(path: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::resolve_path(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn iterdir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::iterdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn glob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::glob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rglob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _build_hash(
    algorithm: &String,
    data: &Vec<u8>,
) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    let alg: String = algorithm.to_lowercase();
    if alg == "md5" {
        return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
            alg,
            (data.clone()).clone(),
            "md5".to_string(),
            SifrInt::from_i64(16),
            SifrInt::from_i64(64),
        );
    } else {
        if alg == "sha1" {
            return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                alg,
                (data.clone()).clone(),
                "sha1".to_string(),
                SifrInt::from_i64(20),
                SifrInt::from_i64(64),
            );
        } else {
            if alg == "sha224" {
                return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                    alg,
                    (data.clone()).clone(),
                    "sha224".to_string(),
                    SifrInt::from_i64(28),
                    SifrInt::from_i64(64),
                );
            } else {
                if alg == "sha256" {
                    return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                        alg,
                        (data.clone()).clone(),
                        "sha256".to_string(),
                        SifrInt::from_i64(32),
                        SifrInt::from_i64(64),
                    );
                } else {
                    if alg == "sha384" {
                        return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                            alg,
                            (data.clone()).clone(),
                            "sha384".to_string(),
                            SifrInt::from_i64(48),
                            SifrInt::from_i64(128),
                        );
                    } else {
                        if alg == "sha512" {
                            return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                                alg,
                                (data.clone()).clone(),
                                "sha512".to_string(),
                                SifrInt::from_i64(64),
                                SifrInt::from_i64(128),
                            );
                        } else {
                            if alg == "blake2b" {
                                return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                                    alg,
                                    (data.clone()).clone(),
                                    "blake2b".to_string(),
                                    SifrInt::from_i64(64),
                                    SifrInt::from_i64(128),
                                );
                            } else {
                                if alg == "blake2s" {
                                    return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                                        alg,
                                        (data.clone()).clone(),
                                        "blake2s".to_string(),
                                        SifrInt::from_i64(32),
                                        SifrInt::from_i64(64),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
        alg,
        (data.clone()).clone(),
        "unknown".to_string(),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
    )
}
fn _is_supported_algorithm(name: &String) -> bool {
    let n: String = name.to_lowercase();
    (((((((n == "md5") || (n == "sha1")) || (n == "sha224")) || (n == "sha256"))
        || (n == "sha384")) || (n == "sha512")) || (n == "blake2b")) || (n == "blake2s")
}
fn _hash_bytes(algorithm: &String, data: &Vec<u8>) -> Vec<u8> {
    if (algorithm).as_str() == "md5" {
        return md5_bytes(data);
    } else {
        if (algorithm).as_str() == "sha1" {
            return sha1_bytes(data);
        } else {
            if (algorithm).as_str() == "sha224" {
                return sha224_bytes(data);
            } else {
                if (algorithm).as_str() == "sha256" {
                    return sha256_bytes(data);
                } else {
                    if (algorithm).as_str() == "sha384" {
                        return sha384_bytes(data);
                    } else {
                        if (algorithm).as_str() == "sha512" {
                            return sha512_bytes(data);
                        } else {
                            if (algorithm).as_str() == "blake2b" {
                                return blake2b_bytes(data);
                            } else {
                                if (algorithm).as_str() == "blake2s" {
                                    return blake2s_bytes(data);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    {
        let __sifr_empty_bytes_literal: Vec<u8> = vec![];
        __sifr_empty_bytes_literal
    }
}
fn _hash_hex(algorithm: &String, data: &Vec<u8>) -> String {
    {
        let __bytes_receiver = &_hash_bytes(algorithm, data);
        let mut __hex = String::with_capacity(
            __bytes_receiver.len().saturating_mul(2_usize),
        );
        for __byte in __bytes_receiver.iter() {
            __hex.push_str(&format!("{:02x}", * __byte));
        }
        __hex
    }
}
fn new(
    name: &String,
    data: &Vec<u8>,
) -> Result<__SifrStdlib_sifr_x2ehashlib_x2eHashObject, ValueError> {
    if !(_is_supported_algorithm(name)) {
        return Err(
            ValueError::new({
                let mut __sifr_concat: String = String::with_capacity(
                    28usize + name.len(),
                );
                __sifr_concat.push_str("unsupported hash algorithm: ");
                __sifr_concat.push_str((name).as_str());
                __sifr_concat
            }),
        );
    }
    Ok(_build_hash(name, data))
}
const PI: f64 = 3.141592653589793_f64;
const E: f64 = 2.718281828459045_f64;
const TAU: f64 = 6.283185307179586_f64;
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
fn ceil(x: f64) -> SifrInt {
    ::sifr_stdlib::math::ceil(x).into_sifr_int()
}
fn log(x: f64) -> f64 {
    ::sifr_stdlib::math::log(x)
}
fn cbrt(x: f64) -> f64 {
    ::sifr_stdlib::math::cbrt(x)
}
fn sin(x: f64) -> f64 {
    ::sifr_stdlib::math::sin(x)
}
fn cos(x: f64) -> f64 {
    ::sifr_stdlib::math::cos(x)
}
fn tan(x: f64) -> f64 {
    ::sifr_stdlib::math::tan(x)
}
fn pow_val(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::pow_val(x, y)
}
fn min_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::min_val(a, b)
}
fn max_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::max_val(a, b)
}
fn round_val(x: f64) -> SifrInt {
    ::sifr_stdlib::math::round_val(x).into_sifr_int()
}
fn asin(x: f64) -> f64 {
    ::sifr_stdlib::math::asin(x)
}
fn acos(x: f64) -> f64 {
    ::sifr_stdlib::math::acos(x)
}
fn atan(x: f64) -> f64 {
    ::sifr_stdlib::math::atan(x)
}
fn atan2(y: f64, x: f64) -> f64 {
    ::sifr_stdlib::math::atan2(y, x)
}
fn sinh(x: f64) -> f64 {
    ::sifr_stdlib::math::sinh(x)
}
fn cosh(x: f64) -> f64 {
    ::sifr_stdlib::math::cosh(x)
}
fn tanh(x: f64) -> f64 {
    ::sifr_stdlib::math::tanh(x)
}
fn log10(x: f64) -> f64 {
    ::sifr_stdlib::math::log10(x)
}
fn log2(x: f64) -> f64 {
    ::sifr_stdlib::math::log2(x)
}
fn exp2(x: f64) -> f64 {
    ::sifr_stdlib::math::exp2(x)
}
fn degrees(x: f64) -> f64 {
    ::sifr_stdlib::math::degrees(x)
}
fn radians(x: f64) -> f64 {
    ::sifr_stdlib::math::radians(x)
}
fn isnan(x: f64) -> bool {
    ::sifr_stdlib::math::isnan(x)
}
fn isinf(x: f64) -> bool {
    ::sifr_stdlib::math::isinf(x)
}
fn trunc(x: f64) -> SifrInt {
    ::sifr_stdlib::math::trunc(x).into_sifr_int()
}
fn copysign(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::copysign(x, y)
}
fn signbit(x: f64) -> bool {
    ::sifr_stdlib::math::signbit(x)
}
fn fmod(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmod(x, y)
}
fn remainder(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::remainder(x, y)
}
fn hypot(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::hypot(x, y)
}
fn fma(x: f64, y: f64, z: f64) -> f64 {
    ::sifr_stdlib::math::fma(x, y, z)
}
fn fmax(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmax(x, y)
}
fn fmin(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmin(x, y)
}
fn exp(x: f64) -> f64 {
    ::sifr_stdlib::math::exp(x)
}
fn expm1(x: f64) -> f64 {
    ::sifr_stdlib::math::expm1(x)
}
fn log1p(x: f64) -> f64 {
    ::sifr_stdlib::math::log1p(x)
}
fn fabs(x: f64) -> f64 {
    ::sifr_stdlib::math::fabs(x)
}
fn isfinite(x: f64) -> bool {
    ::sifr_stdlib::math::isfinite(x)
}
fn isnormal(x: f64) -> bool {
    ::sifr_stdlib::math::isnormal(x)
}
fn issubnormal(x: f64) -> bool {
    ::sifr_stdlib::math::issubnormal(x)
}
fn acosh(x: f64) -> f64 {
    ::sifr_stdlib::math::acosh(x)
}
fn asinh(x: f64) -> f64 {
    ::sifr_stdlib::math::asinh(x)
}
fn atanh(x: f64) -> f64 {
    ::sifr_stdlib::math::atanh(x)
}
fn isqrt(n: SifrInt) -> SifrInt {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .into_sifr_int()
}
fn dist_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::dist(p, q)
}
fn fsum_impl(data: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::fsum(data)
}
fn sumprod_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::sumprod(p, q)
}
fn erf(x: f64) -> f64 {
    ::sifr_stdlib::math::erf(x)
}
fn erfc(x: f64) -> f64 {
    ::sifr_stdlib::math::erfc(x)
}
fn gamma(x: f64) -> f64 {
    ::sifr_stdlib::math::gamma(x)
}
fn lgamma(x: f64) -> f64 {
    ::sifr_stdlib::math::lgamma(x)
}
fn frexp(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::frexp(x)
}
fn ldexp(m: f64, e: SifrInt) -> f64 {
    ::sifr_stdlib::math::ldexp(m, ::sifr_runtime::interop::SifrIntBridge::from(e))
}
fn modf(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::modf(x)
}
fn nextafter(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::nextafter(x, y)
}
fn ulp(x: f64) -> f64 {
    ::sifr_stdlib::math::ulp(x)
}
fn factorial(n: SifrInt) -> SifrInt {
    if &n < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(2);
    while &i <= &n {
        result = &result * &i;
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn gcd(a: SifrInt, b: SifrInt) -> SifrInt {
    let mut x: SifrInt = a.clone();
    let mut y: SifrInt = b.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    while &y != &SifrInt::from_i64(0) {
        let temp: SifrInt = y.clone();
        y = x.floor_mod_known_nonzero(&y);
        x = temp;
    }
    x.clone()
}
fn lcm(a: SifrInt, b: SifrInt) -> SifrInt {
    if &a == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &b == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let g: SifrInt = gcd((a).clone(), (b).clone());
    if &g == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut x: SifrInt = a.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    let mut y: SifrInt = b.clone();
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    &x.floor_div_known_nonzero(&g) * &y
}
fn comb(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    if &k == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    if &k == &n {
        return SifrInt::from_i64(1);
    }
    let mut r: SifrInt = k.clone();
    if &r > &(&n - &k) {
        r = &n - &k;
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &r {
        result = &result * &(&n - &i);
        let divisor: SifrInt = &i + &SifrInt::from_i64(1);
        if &divisor == &SifrInt::from_i64(0) {
            return SifrInt::from_i64(0);
        }
        result = result.floor_div_known_nonzero(&divisor);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn perm(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &k {
        result = &result * &(&n - &i);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn log_base(x: f64, base: f64) -> f64 {
    log(x) / log(base)
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0_f64) {
        return false;
    }
    if abs_tol < (0.0_f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if isnan(a) || isnan(b) {
        return false;
    }
    if isinf(a) || isinf(b) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0_f64) {
        diff = (0.0_f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0_f64) {
        a_abs = (0.0_f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0_f64) {
        b_abs = (0.0_f64) - b_abs;
    }
    let mut larger_abs: f64 = a_abs;
    if b_abs > larger_abs {
        larger_abs = b_abs;
    }
    let mut rel_bound: f64 = rel_tol * larger_abs;
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    diff <= rel_bound
}
fn prod(data: &Vec<SifrInt>) -> SifrInt {
    let mut result: SifrInt = SifrInt::from_i64(1);
    for val in data.iter().cloned() {
        result = &result * &val;
    }
    result.clone()
}
fn _copy_float_list(data: &Vec<f64>) -> Vec<f64> {
    let mut out: Vec<f64> = vec![];
    for value in data.iter().copied() {
        out.push(value);
    }
    out
}
fn dist(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    dist_impl(_copy_float_list(p), _copy_float_list(q))
}
fn fsum(data: &Vec<f64>) -> f64 {
    fsum_impl(_copy_float_list(data))
}
fn sumprod(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    sumprod_impl(_copy_float_list(p), _copy_float_list(q))
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = frexp(x);
    let m: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(0);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> SifrInt {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(1);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return SifrInt::from_i64(0);
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(0);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(1);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}
type __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = ::sifr_runtime::interop::Handle<
    ::sifr_stdlib::regex::CompiledPattern,
>;
trait __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods {
    fn search(&self, text: &String) -> Result<Option<String>, RegexError>;
    fn is_match(&self, text: &String) -> Result<bool, RegexError>;
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError>;
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn pattern(&self) -> Result<String, RegexError>;
    fn flags(&self) -> Result<SifrInt, RegexError>;
}
impl __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods
for __SifrStdlib___sifr_x2eregex_x2eCompiledPattern {
    fn search(&self, text: &String) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_search(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn is_match(&self, text: &String) -> Result<bool, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_is_match(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_replace(self, replacement, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_findall(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_split(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn pattern(&self) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_source(self)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn flags(&self) -> Result<SifrInt, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_flags(self)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
}
fn compile_pattern(
    pattern: &String,
) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
    ::sifr_stdlib::regex::compile_pattern(pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn compile_pattern_flags(
    pattern: &String,
    flags: SifrInt,
) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
    ::sifr_stdlib::regex::compile_pattern_flags(
            pattern,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_match(pattern: &String, text: &String) -> Result<bool, RegexError> {
    ::sifr_stdlib::regex::re_match(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find(pattern: &String, text: &String) -> Result<Option<String>, RegexError> {
    ::sifr_stdlib::regex::re_find(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_replace(
    pattern: &String,
    replacement: &String,
    text: &String,
) -> Result<String, RegexError> {
    ::sifr_stdlib::regex::re_replace(pattern, replacement, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_findall(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_findall(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_split(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_split(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_start(pattern: &String, text: &String) -> Result<SifrInt, RegexError> {
    ::sifr_stdlib::regex::re_find_start(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_end(pattern: &String, text: &String) -> Result<SifrInt, RegexError> {
    ::sifr_stdlib::regex::re_find_end(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_match_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
) -> Result<bool, RegexError> {
    ::sifr_stdlib::regex::re_match_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
) -> Result<Option<String>, RegexError> {
    ::sifr_stdlib::regex::re_find_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_replace_flags(
    pattern: &String,
    replacement: &String,
    text: &String,
    flags: SifrInt,
) -> Result<String, RegexError> {
    ::sifr_stdlib::regex::re_replace_flags(
            pattern,
            replacement,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_findall_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_findall_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_split_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_split_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn search(pattern: &String, text: &String) -> Result<Option<String>, RegexError> {
    re_find(pattern, text)
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    message: String,
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("StatisticsError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
fn _sum(data: &Vec<f64>) -> f64 {
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        total += val;
    }
    total
}
fn _float_int(
    value: SifrInt,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let converted: f64 = value
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
            })?;
        Ok(Ok(converted))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
            }
        }
    }
}
fn _divide_by_int(
    numerator: f64,
    denominator: SifrInt,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let divisor: f64 = _float_int((denominator).clone())?;
        Ok(Ok(numerator / divisor))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    }
}
fn mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let count: SifrInt = SifrInt::from(data.len());
    if &count == &SifrInt::from_i64(0) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ),
        );
    }
    let total: f64 = _sum(data);
    _divide_by_int(total, (count).clone())
}
fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
        let __sifr_io_kind = (&e as &dyn ::std::any::Any)
            .downcast_ref::<std::io::Error>()
            .map(::std::io::Error::kind);
        match __sifr_io_kind {
            Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
            Some(::std::io::ErrorKind::PermissionDenied) => {
                "PermissionDenied".to_string()
            }
            Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
            Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
            Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
            Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                "DirectoryNotEmpty".to_string()
            }
            _ => "Other".to_string(),
        }
    };
    IOError { message: msg, kind }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}
impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for Error {}
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
impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Self::new(err.message)
    }
}
impl From<RegexError> for Error {
    fn from(err: RegexError) -> Self {
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
fn main() {
    let dt: __SifrStdlib_sifr_x2edatetime_x2edatetime = __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
        SifrInt::from_i64(2026),
        SifrInt::from_i64(3),
        SifrInt::from_i64(16),
        SifrInt::from_i64(12),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        None,
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(21usize + 0usize);
        __sifr_concat.push_str("datetime.isoformat = "); __sifr_concat.push_str((dt
        .isoformat()).as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let epoch: __SifrStdlib_sifr_x2edatetime_x2edatetime = from_timestamp(
            0.0_f64,
            &None,
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(29usize +
            0usize); __sifr_concat.push_str("datetime.from_timestamp(0) = ");
            __sifr_concat.push_str((epoch.isoformat()).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(16usize +
            0usize); __sifr_concat.push_str("datetime error: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), RegexError> = (|| {
        let found: Option<String> = search(
            &"\\d+".to_string(),
            &"sifr-1200".to_string(),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(12usize +
            0usize); __sifr_concat.push_str("re.search = "); __sifr_concat
            .push_str(((found).map_or("None".to_string().to_string(), | __v |
            format!("{}", __v))).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(10usize +
            0usize); __sifr_concat.push_str("re error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(23usize + 0usize);
        __sifr_concat.push_str("sifr.math.comb(8, 3) = "); __sifr_concat
        .push_str((format!("{}", comb(SifrInt::from_i64(8), SifrInt::from_i64(3))))
        .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(34usize + 0usize);
        __sifr_concat.push_str("sifr.math.isclose(0.1+0.2, 0.3) = "); __sifr_concat
        .push_str((format!("{}", isclose(0.30000000000000004_f64, 0.3_f64,
        0.000000001_f64, 0.0_f64))).as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (||
    {
        let avg: f64 = mean(&vec![2.0_f64, 4.0_f64, 6.0_f64, 8.0_f64])?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(18usize +
            0usize); __sifr_concat.push_str("statistics.mean = "); __sifr_concat
            .push_str((format!("{}", avg)).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(18usize +
            0usize); __sifr_concat.push_str("statistics error: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let h: __SifrStdlib_sifr_x2ehashlib_x2eHashObject = new(
            &"sha256".to_string(),
            &vec![
                104u8, 97u8, 115u8, 104u8, 108u8, 105u8, 98u8, 45u8, 115u8, 97u8, 109u8,
                112u8, 108u8, 101u8
            ],
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(21usize +
            0usize); __sifr_concat.push_str("hashlib.sha256 len = "); __sifr_concat
            .push_str((format!("{}", SifrInt::from(h.hexdigest().chars().count())))
            .as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(15usize +
            0usize); __sifr_concat.push_str("hashlib error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
}

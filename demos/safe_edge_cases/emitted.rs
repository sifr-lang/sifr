// src/main.rs
mod __sifr_project_nominals {
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
                if (&abs_offset < &SifrInt::from_i64(0)) {
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
            let __sifr_checked_read_collection = &month_days;
            let __sifr_checked_read_index = idx.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
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
        while (&i < &end) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_value.len());
                __sifr_chars_value.get(__sifr_string_index_normalized)
            })
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
        let Some(__sifr_checked_value_2) = ({
            let __sifr_string_index = SifrInt::from_i64(4);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(__sifr_checked_value_3) = ({
            let __sifr_string_index = SifrInt::from_i64(7);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(__sifr_checked_value_4) = ({
            let __sifr_string_index = SifrInt::from_i64(10);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(__sifr_checked_value_5) = ({
            let __sifr_string_index = SifrInt::from_i64(13);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(__sifr_checked_value_6) = ({
            let __sifr_string_index = SifrInt::from_i64(16);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        if ((((__sifr_checked_value_2.clone() != "-")
            || (__sifr_checked_value_3.clone() != "-"))
            || (__sifr_checked_value_4.clone() != "T"))
            || (__sifr_checked_value_5.clone() != ":"))
            || (__sifr_checked_value_6.clone() != ":")
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
        if (({
            let __sifr_string_index = SifrInt::from_i64(6);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
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
            if (sign_value == "-") {
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
            if (&microsecond < &SifrInt::from_i64(0)) {
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
                let ch: Option<String> = ({
                    let __sifr_string_source = &self._hex;
                    let __sifr_string_index = i.clone();
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_string_source.chars().count());
                    __sifr_string_source.chars().nth(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                if let Some(ch) = ch {
                    if (ch != "-") {
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
            let marker: Option<String> = ({
                let __sifr_string_source = &self._hex;
                let __sifr_string_index = SifrInt::from_i64(14);
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_string_source.chars().count());
                __sifr_string_source.chars().nth(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IndexError {
        pub message: String,
    }
    impl IndexError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for IndexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IndexError {}
}
pub use __sifr_project_nominals::FloatOverflowError;
pub use __sifr_project_nominals::FloatPrecisionLossError;
pub use __sifr_project_nominals::IndexError;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2edatetime;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2etimezone;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2euuid_x2eUUID;
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
        let __sifr_checked_read_collection = &month_days;
        let __sifr_checked_read_index = idx.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
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
    while (&i < &end) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
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
    let Some(__sifr_checked_value_2) = ({
        let __sifr_string_index = SifrInt::from_i64(4);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(__sifr_checked_value_3) = ({
        let __sifr_string_index = SifrInt::from_i64(7);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(__sifr_checked_value_4) = ({
        let __sifr_string_index = SifrInt::from_i64(10);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(__sifr_checked_value_5) = ({
        let __sifr_string_index = SifrInt::from_i64(13);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(__sifr_checked_value_6) = ({
        let __sifr_string_index = SifrInt::from_i64(16);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    if ((((__sifr_checked_value_2.clone() != "-")
        || (__sifr_checked_value_3.clone() != "-"))
        || (__sifr_checked_value_4.clone() != "T"))
        || (__sifr_checked_value_5.clone() != ":"))
        || (__sifr_checked_value_6.clone() != ":")
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
    if (({
        let __sifr_string_index = SifrInt::from_i64(6);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_text.len());
        __sifr_chars_text.get(__sifr_string_index_normalized)
    })
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
        if (sign_value == "-") {
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
        if (&microsecond < &SifrInt::from_i64(0)) {
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
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
    message: String,
}
impl __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2egraphlib_x2eCycleError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("CycleError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2egraphlib_x2eCycleError {}
fn topological_sort(
    num_nodes: SifrInt,
    from_nodes: &Vec<SifrInt>,
    to_nodes: &Vec<SifrInt>,
) -> Result<Vec<SifrInt>, __SifrStdlib_sifr_x2egraphlib_x2eCycleError> {
    let mut result: Vec<SifrInt> = vec![];
    let mut visited: Vec<SifrInt> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &num_nodes) {
        visited.push(SifrInt::from_i64(0));
        i = &i + &SifrInt::from_i64(1);
    }
    let mut processed: SifrInt = SifrInt::from_i64(0);
    while (&processed < &num_nodes) {
        let mut found_any: bool = false;
        let mut node: SifrInt = SifrInt::from_i64(0);
        while (&SifrInt::from_i64(0) <= &node) && (&node < &SifrInt::from(visited.len()))
        {
            let v: Option<SifrInt> = {
                let __sifr_checked_read_collection = &visited;
                let __sifr_checked_read_index = node.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(v) = v.clone() {
                if (&v == &SifrInt::from_i64(0)) {
                    let mut has_dep: bool = false;
                    let mut j: SifrInt = SifrInt::from_i64(0);
                    while (&j < &SifrInt::from(to_nodes.len())) {
                        let to_val: Option<SifrInt> = {
                            let __sifr_checked_read_collection = &to_nodes;
                            let __sifr_checked_read_index = j.clone();
                            let __sifr_checked_read_normalized = __sifr_checked_read_index
                                .normalize_index_or_len(
                                    __sifr_checked_read_collection.len(),
                                );
                            __sifr_checked_read_collection
                                .get(__sifr_checked_read_normalized)
                                .cloned()
                        };
                        let from_val: Option<SifrInt> = {
                            let __sifr_checked_read_collection = &from_nodes;
                            let __sifr_checked_read_index = j.clone();
                            let __sifr_checked_read_normalized = __sifr_checked_read_index
                                .normalize_index_or_len(
                                    __sifr_checked_read_collection.len(),
                                );
                            __sifr_checked_read_collection
                                .get(__sifr_checked_read_normalized)
                                .cloned()
                        };
                        if let Some(to_val) = to_val.clone() {
                            if let Some(from_val) = from_val.clone() {
                                if (&to_val == &node) {
                                    let dep_v: Option<SifrInt> = {
                                        let __sifr_checked_read_collection = &visited;
                                        let __sifr_checked_read_index = from_val.clone();
                                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                                            .normalize_index_or_len(
                                                __sifr_checked_read_collection.len(),
                                            );
                                        __sifr_checked_read_collection
                                            .get(__sifr_checked_read_normalized)
                                            .cloned()
                                    };
                                    if let Some(dep_v) = dep_v.clone() {
                                        if (&dep_v == &SifrInt::from_i64(0)) {
                                            has_dep = true;
                                        }
                                    }
                                }
                            }
                        }
                        j = &j + &SifrInt::from_i64(1);
                    }
                    if !has_dep {
                        result.push(node.clone());
                        {
                            let __assign_value = SifrInt::from_i64(1);
                            {
                                let __index_raw = node.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(visited.len());
                                if let Some(__elem) = visited.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                        processed = &processed + &SifrInt::from_i64(1);
                        found_any = true;
                    }
                }
            }
            node = &node + &SifrInt::from_i64(1);
        }
        if !found_any {
            return Err(
                __SifrStdlib_sifr_x2egraphlib_x2eCycleError::new(
                    "cycle detected in graph".to_string(),
                ),
            );
        }
    }
    Ok(result)
}
fn is_valid_ipv4(addr: &String) -> bool {
    let parts: Vec<String> = addr
        .split('.')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if &SifrInt::from(parts.len()) != &SifrInt::from_i64(4) {
        return false;
    }
    for part in parts.iter().cloned() {
        let __sifr_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
        let Some(__sifr_checked_value_0) = ({
            let __sifr_string_index = SifrInt::from_i64(0);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_part.len());
            __sifr_chars_part.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return false;
        };
        if (&SifrInt::from(__sifr_chars_part.len()) > &SifrInt::from_i64(3)) {
            return false;
        }
        if (&SifrInt::from(__sifr_chars_part.len()) > &SifrInt::from_i64(1)) {
            let first_digit: Option<String> = ({
                let __sifr_string_index = SifrInt::from_i64(0);
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_part.len());
                __sifr_chars_part.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if (first_digit != None) && (first_digit == Some("0".to_string())) {
                return false;
            }
        }
        let val: SifrInt = _parse_int(&part);
        if (&val < &SifrInt::from_i64(0)) {
            return false;
        }
        if (&val > &SifrInt::from_i64(255)) {
            return false;
        }
    }
    true
}
fn _parse_int(s: &String) -> SifrInt {
    let __sifr_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
    let mut result: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_s.len())) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_s.len());
            __sifr_chars_s.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == "0") {
                result = &result * &SifrInt::from_i64(10);
            } else {
                if (ch == "1") {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(1);
                } else {
                    if (ch == "2") {
                        result = &(&result * &SifrInt::from_i64(10))
                            + &SifrInt::from_i64(2);
                    } else {
                        if (ch == "3") {
                            result = &(&result * &SifrInt::from_i64(10))
                                + &SifrInt::from_i64(3);
                        } else {
                            if (ch == "4") {
                                result = &(&result * &SifrInt::from_i64(10))
                                    + &SifrInt::from_i64(4);
                            } else {
                                if (ch == "5") {
                                    result = &(&result * &SifrInt::from_i64(10))
                                        + &SifrInt::from_i64(5);
                                } else {
                                    if (ch == "6") {
                                        result = &(&result * &SifrInt::from_i64(10))
                                            + &SifrInt::from_i64(6);
                                    } else {
                                        if (ch == "7") {
                                            result = &(&result * &SifrInt::from_i64(10))
                                                + &SifrInt::from_i64(7);
                                        } else {
                                            if (ch == "8") {
                                                result = &(&result * &SifrInt::from_i64(10))
                                                    + &SifrInt::from_i64(8);
                                            } else {
                                                if (ch == "9") {
                                                    result = &(&result * &SifrInt::from_i64(10))
                                                        + &SifrInt::from_i64(9);
                                                } else {
                                                    return -&SifrInt::from_i64(1);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn _ip_to_int_raw(addr: &String) -> SifrInt {
    let parts: Vec<String> = addr
        .split('.')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: SifrInt = SifrInt::from_i64(0);
    for part in parts.iter().cloned() {
        let val: SifrInt = _parse_int(&part);
        result = &(&result * &SifrInt::from_i64(256)) + &val;
    }
    result.clone()
}
fn ip_to_int(addr: &String) -> Result<SifrInt, ValueError> {
    if !is_valid_ipv4(addr) {
        return Err(ValueError::new("invalid IPv4 address".to_string()));
    }
    Ok(_ip_to_int_raw(addr))
}
fn batched<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: SifrInt,
) -> Result<Vec<Vec<T>>, ValueError> {
    if (&n <= &SifrInt::from_i64(0)) {
        return Err(ValueError::new("batched: n must be > 0".to_string()));
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut current_batch: Vec<T> = vec![];
    for value in data.iter().cloned() {
        current_batch.push(value.clone());
        if (&SifrInt::from(current_batch.len()) == &n) {
            result.push(current_batch.clone());
            current_batch = vec![];
        }
    }
    if (&SifrInt::from(current_batch.len()) > &SifrInt::from_i64(0)) {
        result.push(current_batch.clone());
    }
    Ok(result)
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
    while (&y != &SifrInt::from_i64(0)) {
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
    while (&i < &r) {
        result = &result * &(&n - &i);
        let divisor: SifrInt = &i + &SifrInt::from_i64(1);
        if (&divisor == &SifrInt::from_i64(0)) {
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
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> SifrInt {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(exp_val) = exp_val else {
        return SifrInt::from_i64(0);
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}
fn __const__MT_N() -> SifrInt {
    SifrInt::from_i64(624)
}
fn __const__MT_M() -> SifrInt {
    SifrInt::from_i64(397)
}
fn __const__MT_MATRIX_A() -> SifrInt {
    SifrInt::from_i64(2567483615)
}
fn __const__MT_UPPER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483648)
}
fn __const__MT_LOWER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483647)
}
fn __const__MT_F() -> SifrInt {
    SifrInt::from_i64(1812433253)
}
fn __const__MT_WORD_MASK() -> SifrInt {
    SifrInt::from_i64(4294967295)
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandomState {
    version: SifrInt,
    state_words: Vec<SifrInt>,
    index: SifrInt,
    gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {
    fn new(
        version: SifrInt,
        state_words: Vec<SifrInt>,
        index: SifrInt,
        gauss_next: Option<f64>,
    ) -> Self {
        let __sifr_field_init_0: SifrInt = version.clone();
        let __sifr_field_init_1: Vec<SifrInt> = state_words;
        let __sifr_field_init_2: SifrInt = index.clone();
        let __sifr_field_init_3: Option<f64> = gauss_next;
        Self {
            version: __sifr_field_init_0,
            state_words: __sifr_field_init_1,
            index: __sifr_field_init_2,
            gauss_next: __sifr_field_init_3,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandom {
    _state_words: Vec<SifrInt>,
    _index: SifrInt,
    _gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn new(seed_value: Option<SifrInt>) -> Self {
        let normalized_seed: SifrInt = _normalize_seed_input((seed_value).clone());
        let __sifr_field_init_0: Vec<SifrInt> = _seed_words_from_seed(
            (normalized_seed).clone(),
        );
        let __sifr_field_init_1: SifrInt = __const__MT_N().clone();
        let __sifr_field_init_2: Option<f64> = None;
        Self {
            _state_words: __sifr_field_init_0,
            _index: __sifr_field_init_1,
            _gauss_next: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn seed(&mut self, seed_value: &Option<SifrInt>) {
        let normalized_seed: SifrInt = _normalize_seed_input(
            (seed_value.clone()).clone(),
        );
        self._state_words = _seed_words_from_seed((normalized_seed).clone());
        self._index = __const__MT_N().clone();
        self._gauss_next = None;
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _twist(&mut self) {
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&SifrInt::from_i64(0) <= &i)
            && (&i < &SifrInt::from(self._state_words.len()))
        {
            let y: SifrInt = &(&_state_word_at(&self._state_words, (i).clone())
                & &__const__MT_UPPER_MASK())
                + &(&_state_word_at(
                    &self._state_words,
                    (&i + &SifrInt::from_i64(1))
                        .floor_mod_known_nonzero(&__const__MT_N()),
                ) & &__const__MT_LOWER_MASK());
            let mut x_a: SifrInt = y.floor_div_known_nonzero(&SifrInt::from_i64(2));
            if (&y.floor_mod_known_nonzero(&SifrInt::from_i64(2))
                != &SifrInt::from_i64(0))
            {
                x_a = &x_a ^ &__const__MT_MATRIX_A();
            }
            let new_word: SifrInt = &_state_word_at(
                &self._state_words,
                (&i + &__const__MT_M()).floor_mod_known_nonzero(&__const__MT_N()),
            ) ^ &x_a;
            {
                let __assign_value = &new_word & &__const__MT_WORD_MASK();
                {
                    let __index_raw = i.clone();
                    let __index_normalized = __index_raw
                        .normalize_index_or_len(self._state_words.len());
                    if let Some(__elem) = self._state_words.get_mut(__index_normalized) {
                        *__elem = __assign_value;
                    }
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        self._index = SifrInt::from_i64(0);
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _next_u32(&mut self) -> SifrInt {
        if (&self._index.clone() >= &__const__MT_N()) {
            self._twist();
        }
        let mut y: SifrInt = _state_word_at(&self._state_words, self._index.clone());
        self._index = &self._index.clone() + &SifrInt::from_i64(1);
        y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(2048));
        y = &y ^ &(&(&y * &SifrInt::from_i64(128)) & &SifrInt::from_i64(2636928640));
        y = &y ^ &(&(&y * &SifrInt::from_i64(32768)) & &SifrInt::from_i64(4022730752));
        y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(262144));
        &y & &__const__MT_WORD_MASK()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn random(&mut self) -> f64 {
        random_word_to_unit_float(self._next_u32())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn uniform(&mut self, minimum: f64, maximum: f64) -> f64 {
        minimum + ((maximum - minimum) * self.random())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randrange(
        &mut self,
        start: &SifrInt,
        stop: &Option<SifrInt>,
        step: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if (&step.clone() == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let mut actual_start: SifrInt = start.clone();
        let mut actual_stop: SifrInt = start.clone();
        if (stop.clone() == None) {
            actual_start = SifrInt::from_i64(0);
        } else {
            if let Some(stop) = stop.as_ref() {
                actual_stop = stop.clone();
            }
        }
        let width: SifrInt = &actual_stop - &actual_start;
        if (&step.clone() > &SifrInt::from_i64(0)) {
            if (&width <= &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        } else {
            if (&width >= &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        }
        let mut abs_width: SifrInt = width.clone();
        if &abs_width < &SifrInt::from_i64(0) {
            abs_width = &SifrInt::from_i64(0) - &abs_width;
        }
        let mut abs_step: SifrInt = step.clone();
        if &abs_step < &SifrInt::from_i64(0) {
            abs_step = &SifrInt::from_i64(0) - &abs_step;
        }
        if (&abs_step == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let count: SifrInt = (&(&abs_width + &abs_step) - &SifrInt::from_i64(1))
            .floor_div_known_nonzero(&abs_step);
        if (&count <= &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        if (&count == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        let pick: SifrInt = self._next_u32().floor_mod_known_nonzero(&count);
        Ok(&actual_start + &(&pick * step))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randint(
        &mut self,
        minimum: &SifrInt,
        maximum: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if *minimum > *maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        self.randrange(
            minimum,
            &Some((maximum + &SifrInt::from_i64(1)).clone()),
            &SifrInt::from_i64(1),
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getrandbits(&mut self, k: &SifrInt) -> Result<SifrInt, ValueError> {
        if (&k.clone() < &SifrInt::from_i64(0)) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut bits_left: SifrInt = k.clone();
        while (&bits_left > &SifrInt::from_i64(0)) {
            let word: SifrInt = self._next_u32();
            let mut take: SifrInt = SifrInt::from_i64(32);
            if (&bits_left < &SifrInt::from_i64(32)) {
                take = bits_left.clone();
            }
            let mut mask: SifrInt = SifrInt::from_i64(0);
            let mut shifted_result: SifrInt = result;
            let mut shift_index: SifrInt = SifrInt::from_i64(0);
            while (&shift_index < &take) {
                mask = &(&mask * &SifrInt::from_i64(2)) + &SifrInt::from_i64(1);
                shifted_result = &shifted_result * &SifrInt::from_i64(2);
                shift_index = &shift_index + &SifrInt::from_i64(1);
            }
            result = &shifted_result | &(&word & &mask);
            bits_left = &bits_left - &take;
        }
        Ok(result.clone())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randbytes(&mut self, n: &SifrInt) -> Result<Vec<u8>, ValueError> {
        if (&n.clone() < &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<SifrInt> = vec![];
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *n {
            let byte_value: SifrInt = &self._next_u32() & &SifrInt::from_i64(255);
            values.push(byte_value.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                __out
                    .push(
                        __pair
                            .1
                            .try_to_u8()
                            .map_err(|_error| ValueError {
                                message: format!(
                                    "byte out of range at index {}: {}", __pair.0, * __pair.1
                                ),
                            })?,
                    );
            }
            Ok::<Vec<u8>, ValueError>(__out)
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        let cached: Option<f64> = self._gauss_next;
        if let Some(cached) = cached {
            self._gauss_next = None;
            return mu + (sigma * cached);
        }
        let mut u1: f64 = self.random();
        if u1 <= (0.0_f64) {
            u1 = 0.000000000001_f64;
        }
        let u2: f64 = self.random();
        let radius: f64 = sqrt(-(2.0_f64) * log(u1));
        let theta: f64 = ((2.0_f64) * PI) * u2;
        let z0: f64 = radius * cos(theta);
        let z1: f64 = radius * sin(theta);
        let next_cached: Option<f64> = Some(z1);
        self._gauss_next = next_cached;
        mu + (sigma * z0)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getstate(&self) -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
        __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
            SifrInt::from_i64(3),
            _clone_words(&self._state_words),
            self._index.clone(),
            self._gauss_next,
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn setstate(
        &mut self,
        state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
    ) -> Result<(), ValueError> {
        if (&state.version.clone() != &SifrInt::from_i64(3)) {
            return Err(ValueError::new("setstate: unsupported version".to_string()));
        }
        if (&SifrInt::from(state.state_words.len()) != &__const__MT_N()) {
            return Err(
                ValueError::new("setstate: state_words must have length 624".to_string()),
            );
        }
        if (&state.index.clone() < &SifrInt::from_i64(0))
            || (&state.index.clone() > &__const__MT_N())
        {
            return Err(
                ValueError::new("setstate: index must be in range [0, 624]".to_string()),
            );
        }
        let mut normalized: Vec<SifrInt> = vec![];
        for word in state.state_words.clone().iter().cloned() {
            if (&word < &SifrInt::from_i64(0)) || (&word > &__const__MT_WORD_MASK()) {
                return Err(ValueError::new("setstate: word out of range".to_string()));
            }
            normalized.push(&word & &__const__MT_WORD_MASK());
        }
        self._state_words = normalized;
        self._index = state.index.clone();
        self._gauss_next = state.gauss_next;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2erandom_x2eSystemRandom {}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn new() -> Self {
        Self {}
    }
}
impl ::std::default::Default for __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn default() -> Self {
        Self::new()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn seed(&self, _seed_value: &Option<SifrInt>) {}
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn getstate(
        &self,
    ) -> Result<__SifrStdlib_sifr_x2erandom_x2eRandomState, ValueError> {
        Err(ValueError::new("SystemRandom does not support getstate".to_string()))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn setstate(
        &self,
        _state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
    ) -> Result<(), ValueError> {
        Err(ValueError::new("SystemRandom does not support setstate".to_string()))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn random(&self) -> f64 {
        random_float()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn uniform(&self, minimum: f64, maximum: f64) -> f64 {
        random_uniform(minimum, maximum)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randrange(
        &self,
        start: &SifrInt,
        stop: &Option<SifrInt>,
        step: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        let mut actual_start: SifrInt = start.clone();
        let mut actual_stop: SifrInt = start.clone();
        if (stop.clone() == None) {
            actual_start = SifrInt::from_i64(0);
        } else {
            if let Some(stop) = stop.as_ref() {
                actual_stop = stop.clone();
            }
        }
        random_randrange(
            (actual_start).clone(),
            (actual_stop).clone(),
            (step.clone()).clone(),
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randint(
        &self,
        minimum: &SifrInt,
        maximum: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if *minimum > *maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        Ok(random_int((minimum.clone()).clone(), (maximum.clone()).clone()))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn getrandbits(&self, k: &SifrInt) -> Result<SifrInt, ValueError> {
        if (&k.clone() < &SifrInt::from_i64(0)) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *k {
            let mut bit: SifrInt = SifrInt::from_i64(0);
            if (random_float() >= (0.5_f64)) {
                bit = SifrInt::from_i64(1);
            }
            result = &(&result * &SifrInt::from_i64(2)) + &bit;
            i = &i + &SifrInt::from_i64(1);
        }
        Ok(result.clone())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn gauss(&self, mu: f64, sigma: f64) -> f64 {
        random_gauss(mu, sigma)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randbytes(&self, n: &SifrInt) -> Result<Vec<u8>, ValueError> {
        if (&n.clone() < &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<SifrInt> = vec![];
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *n {
            let value: SifrInt = random_int(
                SifrInt::from_i64(0),
                SifrInt::from_i64(255),
            );
            values.push(value.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                __out
                    .push(
                        __pair
                            .1
                            .try_to_u8()
                            .map_err(|_error| ValueError {
                                message: format!(
                                    "byte out of range at index {}: {}", __pair.0, * __pair.1
                                ),
                            })?,
                    );
            }
            Ok::<Vec<u8>, ValueError>(__out)
        }
    }
}
fn _state_word_at(words: &Vec<SifrInt>, index: SifrInt) -> SifrInt {
    let value: Option<SifrInt> = {
        let __sifr_checked_read_collection = &words;
        let __sifr_checked_read_index = index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(value) = value.clone() {
        return value;
    }
    SifrInt::from_i64(0)
}
fn _clone_words(words: &Vec<SifrInt>) -> Vec<SifrInt> {
    let mut copied: Vec<SifrInt> = vec![];
    for word in words.iter().cloned() {
        copied.push(word.clone());
    }
    copied
}
fn _normalize_seed_input(seed_value: Option<SifrInt>) -> SifrInt {
    if let Some(seed_value) = seed_value.clone() {
        return seed_value.clone();
    }
    random_seed()
}
fn _seed_words_from_seed(seed_value: SifrInt) -> Vec<SifrInt> {
    let mut words: Vec<SifrInt> = vec![];
    words.push(&seed_value & &__const__MT_WORD_MASK());
    let mut i: SifrInt = SifrInt::from_i64(1);
    while (&i < &__const__MT_N()) {
        let prev: SifrInt = _state_word_at(&words, &i - &SifrInt::from_i64(1));
        let next_word: SifrInt = &(&(&__const__MT_F()
            * &(&prev ^ &prev.floor_div_known_nonzero(&SifrInt::from_i64(1073741824))))
            + &i) & &__const__MT_WORD_MASK();
        words.push(next_word.clone());
        i = &i + &SifrInt::from_i64(1);
    }
    words
}
fn _build_state_from_module_storage() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
        SifrInt::from_i64(3),
        random_module_state_words(),
        random_module_state_index(),
        random_module_state_gauss_next(),
    )
}
fn _store_state_into_module_storage(state: &__SifrStdlib_sifr_x2erandom_x2eRandomState) {
    let _set_result: Result<(), ValueError> = random_module_set_state(
        &_clone_words(&state.state_words.clone()),
        state.index.clone(),
        state.gauss_next,
    );
    let _ = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<SifrInt> = random_module_state_words();
    if &SifrInt::from(words.len()) == &__const__MT_N() {
        return;
    }
    let bootstrap: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(5489)),
    );
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> __SifrStdlib_sifr_x2erandom_x2eRandom {
    _ensure_module_state_initialized();
    let mut r: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(0)),
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _set_result: Result<(), ValueError> = r
            .setstate(&_build_state_from_module_storage());
        let _ = _set_result;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    r
}
fn _sync_module_random(generator: &mut __SifrStdlib_sifr_x2erandom_x2eRandom) {
    _store_state_into_module_storage(&generator.getstate());
}
fn seed(seed_value: Option<SifrInt>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        (seed_value).clone(),
    );
    _sync_module_random(&mut generator);
}
fn getstate() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    _ensure_module_state_initialized();
    _build_state_from_module_storage()
}
fn setstate(
    state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
) -> Result<(), ValueError> {
    let mut probe: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(0)),
    );
    let result: Result<(), ValueError> = probe.setstate(state);
    _sync_module_random(&mut probe);
    result
}
fn randint(minimum: SifrInt, maximum: SifrInt) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.randint(&minimum, &maximum);
    _sync_module_random(&mut generator);
    value
}
fn random() -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.random();
    _sync_module_random(&mut generator);
    value
}
fn uniform(minimum: f64, maximum: f64) -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.uniform(minimum, maximum);
    _sync_module_random(&mut generator);
    value
}
fn randrange(
    start: SifrInt,
    stop: Option<SifrInt>,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.randrange(&start, &stop, &step);
    _sync_module_random(&mut generator);
    value
}
fn getrandbits(k: SifrInt) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.getrandbits(&k);
    _sync_module_random(&mut generator);
    value
}
fn gauss(mu: f64, sigma: f64) -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.gauss(mu, sigma);
    _sync_module_random(&mut generator);
    value
}
fn choice<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
) -> Result<T, ValueError> {
    let item_count: SifrInt = SifrInt::from(items.len());
    if (&item_count == &SifrInt::from_i64(0)) {
        return Err(ValueError::new("choice: items must not be empty".to_string()));
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let index: SifrInt = generator._next_u32().floor_mod_known_nonzero(&item_count);
    let picked: Option<T> = {
        let __sifr_checked_read_collection = &items;
        let __sifr_checked_read_index = index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    _sync_module_random(&mut generator);
    if let Some(picked) = picked {
        return Ok(picked);
    }
    Err(ValueError::new("choice: index out of range".to_string()))
}
fn choices<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    k: SifrInt,
) -> Result<Vec<T>, ValueError> {
    if &k <= &SifrInt::from_i64(0) {
        return Ok(vec![]);
    }
    let item_count: SifrInt = SifrInt::from(items.len());
    if (&item_count == &SifrInt::from_i64(0)) {
        return Err(ValueError::new("choices: items must not be empty".to_string()));
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &k) {
        let index: SifrInt = generator._next_u32().floor_mod_known_nonzero(&item_count);
        let picked: Option<T> = {
            let __sifr_checked_read_collection = &items;
            let __sifr_checked_read_index = index.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone());
        } else {
            return Err(ValueError::new("choices: index out of range".to_string()));
        }
        i = &i + &SifrInt::from_i64(1);
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn sample<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    k: SifrInt,
) -> Result<Vec<T>, ValueError> {
    if (&k < &SifrInt::from_i64(0)) {
        return Err(ValueError::new("sample: k must be >= 0".to_string()));
    }
    if (&k > &SifrInt::from(items.len())) {
        return Err(ValueError::new("sample larger than population".to_string()));
    }
    let mut pool: Vec<T> = vec![];
    for item in items.iter().cloned() {
        pool.push(item.clone());
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut remaining: SifrInt = SifrInt::from(pool.len());
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &k) {
        if (&remaining == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("sample larger than population".to_string()));
        }
        let pick_index: SifrInt = generator
            ._next_u32()
            .floor_mod_known_nonzero(&remaining);
        let picked: Option<T> = {
            let __sifr_checked_read_collection = &pool;
            let __sifr_checked_read_index = pick_index.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone());
        }
        let last: Option<T> = {
            let __sifr_checked_read_collection = &pool;
            let __sifr_checked_read_index = &remaining - &SifrInt::from_i64(1);
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(last) = last {
            if (&SifrInt::from_i64(0) <= &pick_index)
                && (&pick_index < &SifrInt::from(pool.len()))
            {
                {
                    let __assign_value = last.clone();
                    {
                        let __index_raw = pick_index.clone();
                        let __index_normalized = __index_raw
                            .normalize_index_or_len(pool.len());
                        if let Some(__elem) = pool.get_mut(__index_normalized) {
                            *__elem = __assign_value;
                        }
                    }
                }
            }
        }
        remaining = &remaining - &SifrInt::from_i64(1);
        i = &i + &SifrInt::from_i64(1);
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn shuffle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(items: &mut Vec<T>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let n: SifrInt = SifrInt::from(items.len());
    if (&n > &SifrInt::from_i64(1)) {
        let mut i: SifrInt = &n - &SifrInt::from_i64(1);
        while (&i > &SifrInt::from_i64(0)) {
            let divisor: SifrInt = &i + &SifrInt::from_i64(1);
            if (&divisor == &SifrInt::from_i64(0)) {
                return;
            }
            let j: SifrInt = generator._next_u32().floor_mod_known_nonzero(&divisor);
            let left: Option<T> = {
                let __sifr_checked_read_collection = &items;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let right: Option<T> = {
                let __sifr_checked_read_collection = &items;
                let __sifr_checked_read_index = j.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(left) = left {
                if let Some(right) = right {
                    if (&SifrInt::from_i64(0) <= &i)
                        && (&i < &SifrInt::from(items.len()))
                    {
                        {
                            let __assign_value = right.clone();
                            {
                                let __index_raw = i.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(items.len());
                                if let Some(__elem) = items.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                    if (&SifrInt::from_i64(0) <= &j)
                        && (&j < &SifrInt::from(items.len()))
                    {
                        {
                            let __assign_value = left.clone();
                            {
                                let __index_raw = j.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(items.len());
                                if let Some(__elem) = items.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
    }
    _sync_module_random(&mut generator);
}
fn randbytes(n: SifrInt) -> Result<Vec<u8>, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<Vec<u8>, ValueError> = generator.randbytes(&n);
    _sync_module_random(&mut generator);
    value
}
fn randbelow(n: SifrInt) -> Result<SifrInt, ValueError> {
    if (&n <= &SifrInt::from_i64(0)) {
        return Err(ValueError::new("randbelow: n must be > 0".to_string()));
    }
    Ok(random_int(SifrInt::from_i64(0), &n - &SifrInt::from_i64(1)))
}
fn _replace_whitespace_chars(text: &String, replace_tabs: bool) -> String {
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
fn _expand_tabs_impl(text: &String, tabsize: SifrInt) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut effective_tabsize: SifrInt = tabsize.clone();
    if &effective_tabsize <= &SifrInt::from_i64(0) {
        effective_tabsize = SifrInt::from_i64(1);
    }
    if (&effective_tabsize == &SifrInt::from_i64(0)) {
        return text.clone();
    }
    let mut result: String = "".to_string();
    let mut column: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch_opt: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if (ch == "\t") {
                let mut spaces: SifrInt = &effective_tabsize
                    - &column.floor_mod_known_nonzero(&effective_tabsize);
                if (&spaces <= &SifrInt::from_i64(0)) {
                    spaces = effective_tabsize.clone();
                }
                let mut j: SifrInt = SifrInt::from_i64(0);
                while (&j < &spaces) {
                    result.push(' ');
                    j = &j + &SifrInt::from_i64(1);
                }
                column = &column + &spaces;
            } else {
                if (ch == "\n") || (ch == "\r") {
                    result.push_str((ch).as_str());
                    column = SifrInt::from_i64(0);
                } else {
                    result.push_str((ch).as_str());
                    column = &column + &SifrInt::from_i64(1);
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _prepare_text(
    text: &String,
    expand_tabs: bool,
    tabsize: SifrInt,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = {
        let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
        __sifr_concat.push_str((text).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if expand_tabs {
        prepared = _expand_tabs_impl(&prepared, (tabsize).clone());
    }
    if replace_whitespace {
        prepared = _replace_whitespace_chars(&prepared, true);
    }
    prepared
}
fn _normalize_whitespace(text: &String) -> String {
    _prepare_text(text, true, SifrInt::from_i64(8), true)
}
fn _split_word_units(word: &String, break_on_hyphens: bool) -> Vec<String> {
    if !break_on_hyphens {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str((word).as_str()); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let parts: Vec<String> = word
        .split('-')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (&SifrInt::from(parts.len()) <= &SifrInt::from_i64(1)) {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str((word).as_str()); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let mut units: Vec<String> = vec![];
    let mut index: SifrInt = SifrInt::from_i64(0);
    for part in parts.iter().cloned() {
        let __sifr_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
        let is_last: bool = (&index
            == &(&SifrInt::from(parts.len()) - &SifrInt::from_i64(1)));
        if is_last {
            if (&SifrInt::from(__sifr_chars_part.len()) > &SifrInt::from_i64(0)) {
                units.push(part.clone());
            }
        } else {
            if (&SifrInt::from(__sifr_chars_part.len()) == &SifrInt::from_i64(0)) {
                units.push("-".to_string());
            } else {
                units.push(format!("{}{}", part, "-"));
            }
        }
        index = &index + &SifrInt::from_i64(1);
    }
    if (&SifrInt::from(units.len()) == &SifrInt::from_i64(0)) {
        units.push(format!("{}{}", word, ""));
    }
    units
}
fn _trim_line(line: &String) -> String {
    let __sifr_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
    let mut start: SifrInt = SifrInt::from_i64(0);
    while (&start < &SifrInt::from(__sifr_chars_line.len()))
        && ({
            let __sifr_string_index = start.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_line.len());
            __sifr_chars_line.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string())
            .is_some_and(|__sifr_checked_value_2| {
                (__sifr_checked_value_2.clone() == " ")
            })
    {
        start = &start + &SifrInt::from_i64(1);
    }
    let mut end: SifrInt = SifrInt::from(__sifr_chars_line.len());
    while (&end > &start)
        && (({
            let __sifr_string_index = &end - &SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_line.len());
            __sifr_chars_line.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) == Some(" ".to_string()))
    {
        end = &end - &SifrInt::from_i64(1);
    }
    {
        let _slice_src = &__sifr_chars_line;
        let _slice_len = _slice_src.len();
        let _slice_start = start.clamp_slice_bound(_slice_len);
        let _slice_stop = end.clamp_slice_bound(_slice_len);
        String::from_iter(
            _slice_src
                .iter()
                .skip(_slice_start)
                .take(_slice_stop.saturating_sub(_slice_start))
                .copied(),
        )
    }
}
fn _finalize_line(line: &String, drop_whitespace: bool) -> String {
    if drop_whitespace {
        return _trim_line(line);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(line.len() + 0usize);
        __sifr_concat.push_str((line).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _wrap_impl(text: &String, width: SifrInt) -> Vec<String> {
    let normalized: String = _normalize_whitespace(text);
    _wrap_with_indents(
        &normalized,
        (width).clone(),
        &"".to_string(),
        &"".to_string(),
        true,
        true,
    )
}
fn _effective_content_width(total_width: SifrInt, indent: &String) -> SifrInt {
    let __sifr_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
    let available: SifrInt = &total_width - &SifrInt::from(__sifr_chars_indent.len());
    if &available <= &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    available.clone()
}
fn _push_current_line(
    result: &mut Vec<String>,
    line: &String,
    indent: &String,
    drop_whitespace: bool,
) {
    let candidate: String = _finalize_line(
        &format!("{}{}", indent, line),
        drop_whitespace,
    );
    let __sifr_chars_candidate: Vec<char> = candidate.chars().collect::<Vec<char>>();
    if drop_whitespace {
        if (&SifrInt::from(__sifr_chars_candidate.len()) > &SifrInt::from_i64(0)) {
            result.push(candidate.clone());
        }
    } else {
        result.push(candidate.clone());
    }
}
fn _wrap_with_indents(
    text: &String,
    total_width: SifrInt,
    initial_indent: &String,
    subsequent_indent: &String,
    break_on_hyphens: bool,
    drop_whitespace: bool,
) -> Vec<String> {
    let words: Vec<String> = text
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: Vec<String> = vec![];
    let mut current: String = "".to_string();
    let mut first_line: bool = true;
    let mut current_limit: SifrInt = _effective_content_width(
        (total_width).clone(),
        initial_indent,
    );
    for raw_word in words.iter().cloned() {
        let units: Vec<String> = _split_word_units(&raw_word, break_on_hyphens);
        for word in units.iter().cloned() {
            let __sifr_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
            if (&SifrInt::from(__sifr_chars_word.len()) == &SifrInt::from_i64(0)) {
                if drop_whitespace {
                    continue;
                }
                if (&SifrInt::from(current.chars().count()) > &SifrInt::from_i64(0)) {
                    if (&(&SifrInt::from(current.chars().count())
                        + &SifrInt::from_i64(1)) <= &current_limit)
                    {
                        current.push(' ');
                    }
                }
                continue;
            }
            if (&SifrInt::from(current.chars().count()) == &SifrInt::from_i64(0)) {
                current = word;
            } else {
                if (&(&(&SifrInt::from(current.chars().count()) + &SifrInt::from_i64(1))
                    + &SifrInt::from(__sifr_chars_word.len())) <= &current_limit)
                {
                    current.push(' ');
                    current.push_str((word).as_str());
                } else {
                    if first_line {
                        _push_current_line(
                            &mut result,
                            &current,
                            initial_indent,
                            drop_whitespace,
                        );
                        first_line = false;
                        current_limit = _effective_content_width(
                            (total_width).clone(),
                            subsequent_indent,
                        );
                    } else {
                        _push_current_line(
                            &mut result,
                            &current,
                            subsequent_indent,
                            drop_whitespace,
                        );
                    }
                    current = word;
                }
            }
        }
    }
    if (&SifrInt::from(current.chars().count()) > &SifrInt::from_i64(0)) {
        if first_line {
            _push_current_line(&mut result, &current, initial_indent, drop_whitespace);
        } else {
            _push_current_line(
                &mut result,
                &current,
                subsequent_indent,
                drop_whitespace,
            );
        }
    }
    result
}
fn wrap(text: &String, width: SifrInt) -> Result<Vec<String>, ValueError> {
    if (&width <= &SifrInt::from_i64(0)) {
        return Err(ValueError::new("wrap: width must be > 0".to_string()));
    }
    Ok(_wrap_impl(text, (width).clone()))
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
        let left: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let right: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_prefix.len());
            __sifr_chars_prefix.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
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
        let first: Option<String> = ({
            let __sifr_string_source = &normalized_input;
            let __sifr_string_index = SifrInt::from_i64(0);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
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
    while (&i < &input_len) {
        let ch_opt: Option<String> = ({
            let __sifr_string_source = &normalized_input;
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if (ch == "-") {} else {
                if !_is_hex_char(&ch) {
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
    if (&input_len == &SifrInt::from_i64(36)) {
        let h1: Option<String> = ({
            let __sifr_string_source = &normalized_input;
            let __sifr_string_index = SifrInt::from_i64(8);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let h2: Option<String> = ({
            let __sifr_string_source = &normalized_input;
            let __sifr_string_index = SifrInt::from_i64(13);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let h3: Option<String> = ({
            let __sifr_string_source = &normalized_input;
            let __sifr_string_index = SifrInt::from_i64(18);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let h4: Option<String> = ({
            let __sifr_string_source = &normalized_input;
            let __sifr_string_index = SifrInt::from_i64(23);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if (((h1 != Some("-".to_string())) || (h2 != Some("-".to_string())))
            || (h3 != Some("-".to_string()))) || (h4 != Some("-".to_string()))
        {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    } else {
        if (&input_len != &SifrInt::from_i64(32)) {
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
        let part: Option<String> = ({
            let __sifr_string_source = &hex_only;
            let __sifr_string_index = j.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(part) = part {
            canonical.push_str((part).as_str());
        }
        j = &j + &SifrInt::from_i64(1);
    }
    Ok(canonical)
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
impl From<IndexError> for Error {
    fn from(err: IndexError) -> Self {
        Self::new(err.message)
    }
}
fn main() {
    println!("=== 1. random.randint: Validates a <= b ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let r: SifrInt = randint(SifrInt::from_i64(1), SifrInt::from_i64(10))?;
        println!("randint(1, 10) = ok");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let r2: SifrInt = randint(SifrInt::from_i64(5), SifrInt::from_i64(3))?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("randint(5, 3) -> ValueError: {}", e.message.clone());
    }
    println!("=== 2. secrets.randbelow: Validates n > 0 ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let s: SifrInt = randbelow(SifrInt::from_i64(100))?;
        println!("randbelow(100) = ok");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let s2: SifrInt = randbelow(SifrInt::from_i64(0))?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("randbelow(0) -> ValueError: {}", e.message.clone());
    }
    println!("=== 3. textwrap.wrap: Validates width > 0 ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let lines: Vec<String> = wrap(&"hello world".to_string(), SifrInt::from_i64(5))?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity((27usize +
            0usize) + 7usize); __sifr_concat.push_str("wrap(hello world, 5) = ok (");
            __sifr_concat.push_str((format!("{}", SifrInt::from(lines.len()))).as_str());
            __sifr_concat.push_str(" lines)"); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(7usize +
            0usize); __sifr_concat.push_str("error: "); __sifr_concat.push_str((e.message
            .clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let lines2: Vec<String> = wrap(&"hello".to_string(), SifrInt::from_i64(0))?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(30usize +
            0usize); __sifr_concat.push_str("wrap(hello, 0) -> ValueError: ");
            __sifr_concat.push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    println!("=== 4. itertools.batched: Validates n > 0 ===");
    let data: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5)
    ];
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let b: Vec<Vec<SifrInt>> = batched(
            &(data).iter().cloned().collect::<Vec<_>>(),
            SifrInt::from_i64(2),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity((30usize +
            0usize) + 9usize); __sifr_concat.push_str("batched([1,2,3,4,5], 2) = ok (");
            __sifr_concat.push_str((format!("{}", SifrInt::from(b.len()))).as_str());
            __sifr_concat.push_str(" batches)"); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(7usize +
            0usize); __sifr_concat.push_str("error: "); __sifr_concat.push_str((e.message
            .clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let b2: Vec<Vec<SifrInt>> = batched(
            &(data).iter().cloned().collect::<Vec<_>>(),
            SifrInt::from_i64(0),
        )?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(32usize +
            0usize); __sifr_concat.push_str("batched(data, 0) -> ValueError: ");
            __sifr_concat.push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    println!("=== 5. graphlib.topological_sort: Cycle Detection ===");
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2egraphlib_x2eCycleError> = (|| {
        let order: Vec<SifrInt> = topological_sort(
            SifrInt::from_i64(3),
            &vec![SifrInt::from_i64(0), SifrInt::from_i64(0)],
            &vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(15usize +
            0usize); __sifr_concat.push_str("acyclic graph: "); __sifr_concat
            .push_str((format!("{:?}", order)).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(7usize +
            0usize); __sifr_concat.push_str("error: "); __sifr_concat.push_str((e.message
            .clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2egraphlib_x2eCycleError> = (|| {
        let order2: Vec<SifrInt> = topological_sort(
            SifrInt::from_i64(2),
            &vec![SifrInt::from_i64(0), SifrInt::from_i64(1)],
            &vec![SifrInt::from_i64(1), SifrInt::from_i64(0)],
        )?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("cyclic graph -> CycleError: {}", e.message.clone());
    }
    println!("=== 6. uuid.uuid_from_hex: Validates hex format ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let u: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(
            &"550e8400e29b41d4a716446655440000".to_string(),
        )?;
        println!("valid UUID hex: ok");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let u2: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(
            &"xyz-invalid!".to_string(),
        )?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("invalid chars -> ValueError: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let u3: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(
            &"abcd1234".to_string(),
        )?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("wrong length -> ValueError: {}", e.message.clone());
    }
    println!("=== 7. ipaddress.ip_to_int: Validates IPv4 format ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let ip: SifrInt = ip_to_int(&"192.168.1.1".to_string())?;
        println!("ip_to_int(192.168.1.1) = ok");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(7usize +
            0usize); __sifr_concat.push_str("error: "); __sifr_concat.push_str((e.message
            .clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let ip2: SifrInt = ip_to_int(&"bad".to_string())?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(30usize +
            0usize); __sifr_concat.push_str("ip_to_int(bad) -> ValueError: ");
            __sifr_concat.push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    println!("=== 8. datetime.from_timestamp: Validates timestamp ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let dt: __SifrStdlib_sifr_x2edatetime_x2edatetime = from_timestamp(
            0.0_f64,
            &None,
        )?;
        println!("from_timestamp(0.0) = ok");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let dt2: __SifrStdlib_sifr_x2edatetime_x2edatetime = from_timestamp(
            -(99999999999999.0_f64),
            &None,
        )?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("from_timestamp(invalid) -> ValueError: {}", e.message.clone());
    }
    println!("=== 9. SubscriptAssign: Bounds-checked (IndexError) ===");
    let mut nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)
    ];
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(8usize + 0usize);
        __sifr_concat.push_str("before: "); __sifr_concat.push_str((format!("{:?}",
        nums)).as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), IndexError> = (|| {
        {
            let __assign_value = SifrInt::from_i64(999);
            {
                let __index_raw = SifrInt::from_i64(99);
                let __index_normalized = __index_raw.normalize_index_or_len(nums.len());
                if let Some(__elem) = nums.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                } else {
                    return Err(
                        IndexError::new("collection index out of range".to_string()),
                    );
                }
            }
        }
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let _e = __sifr_try_err.clone();
        println!("out-of-bounds assign -> IndexError");
    }
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(27usize + 0usize);
        __sifr_concat.push_str("after out-of-bounds error: "); __sifr_concat
        .push_str((format!("{:?}", nums)).as_str()); __sifr_concat }
    );
    if (&SifrInt::from(nums.len()) > &SifrInt::from_i64(1)) {
        {
            let __assign_value = SifrInt::from_i64(99);
            {
                let __index_raw = SifrInt::from_i64(1);
                let __index_normalized = __index_raw.normalize_index_or_len(nums.len());
                if let Some(__elem) = nums.get_mut(__index_normalized) {
                    *__elem = __assign_value;
                }
            }
        }
    }
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("after valid assign: "); __sifr_concat
        .push_str((format!("{:?}", nums)).as_str()); __sifr_concat }
    );
    println!("demo complete!");
}

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
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2edatetime_x2edate {
        pub year: SifrInt,
        pub month: SifrInt,
        pub day: SifrInt,
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edate {
        pub fn new(year: SifrInt, month: SifrInt, day: SifrInt) -> Self {
            Self { year, month, day }
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edate {
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
            {
                let mut __sifr_concat: String = String::with_capacity(
                    (((y.len() + 1usize) + mo.len()) + 1usize) + d.len(),
                );
                __sifr_concat.push_str((y).as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str((mo).as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str((d).as_str());
                __sifr_concat
            }
        }
    }
    impl PartialEq for __SifrStdlib_sifr_x2edatetime_x2edate {
        fn eq(&self, other: &__SifrStdlib_sifr_x2edatetime_x2edate) -> bool {
            ((((self.year.clone() == other.year.clone()))
                && ((self.month.clone() == other.month.clone())))
                && ((self.day.clone() == other.day.clone())))
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2edate {
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
}
pub use __sifr_project_nominals::FloatOverflowError;
pub use __sifr_project_nominals::FloatPrecisionLossError;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2edate;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2edatetime;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2etimezone;
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
fn now(
    tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
) -> __SifrStdlib_sifr_x2edatetime_x2edatetime {
    let current_epoch: f64 = time_now();
    let __sifr_try_res: Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> = (||
    {
        let current: __SifrStdlib_sifr_x2edatetime_x2edatetime = _from_timestamp_with_tz(
            current_epoch,
            tz,
        )?;
        Ok(current)
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            let parts: Vec<SifrInt> = datetime_now_struct();
            let mut yr: SifrInt = SifrInt::from_i64(0);
            let mut mo: SifrInt = SifrInt::from_i64(1);
            let mut dy: SifrInt = SifrInt::from_i64(1);
            let mut hr: SifrInt = SifrInt::from_i64(0);
            let mut mn: SifrInt = SifrInt::from_i64(0);
            let mut sc: SifrInt = SifrInt::from_i64(0);
            for (i, v) in Box::new(
                (parts)
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|__pair| (
                        SifrInt::from(__pair.0) + SifrInt::from_i64(0),
                        __pair.1,
                    )),
            ) {
                if &i == &SifrInt::from_i64(0) {
                    yr = v.clone();
                }
                if &i == &SifrInt::from_i64(1) {
                    mo = v.clone();
                }
                if &i == &SifrInt::from_i64(2) {
                    dy = v.clone();
                }
                if &i == &SifrInt::from_i64(3) {
                    hr = v.clone();
                }
                if &i == &SifrInt::from_i64(4) {
                    mn = v.clone();
                }
                if &i == &SifrInt::from_i64(5) {
                    sc = v.clone();
                }
            }
            if let Some(tz) = tz.as_ref() {
                let __sifr_try_res: Result<
                    __SifrStdlib_sifr_x2edatetime_x2edatetime,
                    ValueError,
                > = (|| {
                    let parsed_offset: SifrInt = _timezone_offset_from_text(
                        &format!("{}", tz),
                    )?;
                    Ok(
                        __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                            (yr).clone(),
                            (mo).clone(),
                            (dy).clone(),
                            (hr).clone(),
                            (mn).clone(),
                            (sc).clone(),
                            SifrInt::from_i64(0),
                            Some(parsed_offset),
                        ),
                    )
                })();
                match __sifr_try_res {
                    Ok(__sifr_ret_val) => {
                        return __sifr_ret_val;
                    }
                    Err(__sifr_try_err) => {
                        let _e = __sifr_try_err.clone();
                        return __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                            (yr).clone(),
                            (mo).clone(),
                            (dy).clone(),
                            (hr).clone(),
                            (mn).clone(),
                            (sc).clone(),
                            SifrInt::from_i64(0),
                            None,
                        );
                    }
                }
            }
            return __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                (yr).clone(),
                (mo).clone(),
                (dy).clone(),
                (hr).clone(),
                (mn).clone(),
                (sc).clone(),
                SifrInt::from_i64(0),
                None,
            );
        }
    }
}
fn today() -> __SifrStdlib_sifr_x2edatetime_x2edate {
    let current: __SifrStdlib_sifr_x2edatetime_x2edatetime = now(&None);
    __SifrStdlib_sifr_x2edatetime_x2edate::new(
        current.year.clone(),
        current.month.clone(),
        current.day.clone(),
    )
}
fn from_timestamp(
    ts: f64,
    tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
    _from_timestamp_with_tz(ts, tz)
}
fn main() {
    let current: __SifrStdlib_sifr_x2edatetime_x2edatetime = now(&None);
    let current_iso: String = current.isoformat();
    let current_has_t: bool = current_iso.contains(&"T".to_string());
    println!("current_has_t = {}", current_has_t);
    assert!(
        (format!("{}", format!("current_has_t = {}", current_has_t)) ==
        "current_has_t = true")
    );
    let day: __SifrStdlib_sifr_x2edatetime_x2edate = today();
    let today_iso: String = day.isoformat();
    let today_has_dash: bool = today_iso.contains(&"-".to_string());
    println!("today_has_dash = {}", today_has_dash);
    assert!(
        (format!("{}", format!("today_has_dash = {}", today_has_dash)) ==
        "today_has_dash = true")
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let epoch: __SifrStdlib_sifr_x2edatetime_x2edatetime = from_timestamp(
            0.0_f64,
            &None,
        )?;
        let epoch_text: String = epoch.isoformat();
        println!("from_timestamp_ok = {}", epoch_text);
        assert!(
            (format!("{}", format!("from_timestamp_ok = {}", epoch_text)) ==
            "from_timestamp_ok = 1970-01-01T00:00:00")
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("unexpected_error = {}", e.message.clone());
        assert!(
            (format!("{}", format!("unexpected_error = {}", e.message.clone())) ==
            "from_timestamp_invalid = invalid timestamp")
        );
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let bad: __SifrStdlib_sifr_x2edatetime_x2edatetime = from_timestamp(
            -(99999999999999.0_f64),
            &None,
        )?;
        println!("from_timestamp_invalid_unexpected = {}", bad);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("from_timestamp_invalid = {}", e.message.clone());
    }
}

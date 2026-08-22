// src/main.rs
mod __sifr_project_nominals {
    pub fn datetime_now() -> String {
        ::sifr_stdlib::time::datetime_now()
    }
    pub fn datetime_now_struct() -> Vec<i64> {
        ::sifr_stdlib::time::datetime_now_struct()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
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
    pub fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
        ::sifr_stdlib::time::time_strptime(s, fmt)
            .map(|__sifr_bridge_ok| {
                __sifr_bridge_ok
                    .into_iter()
                    .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                    .collect()
            })
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn time_gmtime() -> Vec<i64> {
        ::sifr_stdlib::time::time_gmtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
            .collect()
    }
    pub fn time_localtime() -> Vec<i64> {
        ::sifr_stdlib::time::time_localtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
            .collect()
    }
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub _offset: i64,
    }
    impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub fn new(offset: i64) -> Self {
            let __sifr_field_init_0: i64 = offset;
            Self {
                _offset: __sifr_field_init_0,
            }
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub fn offset(&self) -> i64 {
            self._offset
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub fn iso_suffix(&self) -> String {
            let mut sign: String = "+".to_string();
            if (self._offset < (0_i64)) {
                sign = "-".to_string();
            }
            let mut abs_offset: i64 = self._offset;
            if abs_offset < (0_i64) {
                abs_offset = -abs_offset;
            }
            let h: i64 = abs_offset / (3600_i64);
            let m: i64 = (abs_offset % (3600_i64)) / (60_i64);
            let mut hs: String = format!("{}", h);
            if ((hs.chars().count() as i64) < (2_i64)) {
                hs = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + hs.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((hs).as_str());
                    __sifr_concat
                };
            }
            let mut ms: String = format!("{}", m);
            if ((ms.chars().count() as i64) < (2_i64)) {
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
            if (self._offset == (0_i64)) {
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
        pub year: i64,
        pub month: i64,
        pub day: i64,
        pub hour: i64,
        pub minute: i64,
        pub second: i64,
        pub microsecond: i64,
        pub _tz_offset: Option<i64>,
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn new(
            year: i64,
            month: i64,
            day: i64,
            hour: i64,
            minute: i64,
            second: i64,
            microsecond: i64,
            tz_offset: Option<i64>,
        ) -> Self {
            let __sifr_field_init_0: i64 = year;
            let __sifr_field_init_1: i64 = month;
            let __sifr_field_init_2: i64 = day;
            let __sifr_field_init_3: i64 = hour;
            let __sifr_field_init_4: i64 = minute;
            let __sifr_field_init_5: i64 = second;
            let __sifr_field_init_6: i64 = microsecond;
            let __sifr_field_init_7: Option<i64> = tz_offset;
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
            let y: String = format!("{}", self.year);
            let mut mo: String = format!("{}", self.month);
            if ((mo.chars().count() as i64) < (2_i64)) {
                mo = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mo).as_str());
                    __sifr_concat
                };
            }
            let mut d: String = format!("{}", self.day);
            if ((d.chars().count() as i64) < (2_i64)) {
                d = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((d).as_str());
                    __sifr_concat
                };
            }
            let mut h: String = format!("{}", self.hour);
            if ((h.chars().count() as i64) < (2_i64)) {
                h = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((h).as_str());
                    __sifr_concat
                };
            }
            let mut mi: String = format!("{}", self.minute);
            if ((mi.chars().count() as i64) < (2_i64)) {
                mi = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mi).as_str());
                    __sifr_concat
                };
            }
            let mut s: String = format!("{}", self.second);
            if ((s.chars().count() as i64) < (2_i64)) {
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
            if (self.microsecond != (0_i64)) {
                base.push('.');
                base.push_str((_six_digits(self.microsecond)).as_str());
            }
            let tz_offset_opt: Option<i64> = self._tz_offset;
            if let Some(tz_offset_opt) = tz_offset_opt {
                let offset: i64 = tz_offset_opt;
                let mut sign: String = "+".to_string();
                let mut abs_offset: i64 = offset;
                if abs_offset < (0_i64) {
                    sign = "-".to_string();
                    abs_offset = -abs_offset;
                }
                let h_off: i64 = abs_offset / (3600_i64);
                let m_off: i64 = (abs_offset % (3600_i64)) / (60_i64);
                let mut hs_off: String = format!("{}", h_off);
                if ((hs_off.chars().count() as i64) < (2_i64)) {
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
                if ((ms_off.chars().count() as i64) < (2_i64)) {
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
        pub fn timestamp(&self) -> i64 {
            let mut days: i64 = 0_i64;
            if (self.year >= (1970_i64)) {
                let mut y: i64 = 1970_i64;
                while (y < self.year) {
                    days += _days_in_year(y);
                    y += 1_i64;
                }
            } else {
                let mut y: i64 = 1969_i64;
                while (y >= self.year) {
                    days -= _days_in_year(y);
                    y -= 1_i64;
                }
            }
            let mut m: i64 = 1_i64;
            while (m < self.month) {
                days += _days_in_month(self.year, m);
                m += 1_i64;
            }
            days = (days + self.day) - (1_i64);
            let naive_timestamp: i64 = (((days * (86400_i64)) + (self.hour * (3600_i64)))
                + (self.minute * (60_i64))) + self.second;
            let tz_offset_opt: Option<i64> = self._tz_offset;
            if let Some(tz_offset_opt) = tz_offset_opt {
                let offset: i64 = tz_offset_opt;
                return naive_timestamp - offset;
            }
            naive_timestamp
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn timestamp_microseconds(&self) -> i64 {
            (self.timestamp() * (1000000_i64)) + self.microsecond
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn astimezone(
            &self,
            tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
        ) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
            let mut target: __SifrStdlib_sifr_x2edatetime_x2etimezone = __SifrStdlib_sifr_x2edatetime_x2etimezone::new(
                0_i64,
            );
            if let Some(tz) = tz.as_ref() {
                let __sifr_try_res: Result<(String, i64), ValueError> = (|| {
                    let tz_text: String = format!("{}", tz);
                    let target_offset: i64 = _timezone_offset_from_text(&tz_text)?;
                    target = __SifrStdlib_sifr_x2edatetime_x2etimezone::new(target_offset);
                    Ok((tz_text, target_offset))
                })();
                let (tz_text, target_offset) = match __sifr_try_res {
                    Ok(__sifr_try_bindings) => __sifr_try_bindings,
                    Err(__sifr_try_err) => {
                        let e = __sifr_try_err.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                };
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
            (((((((((self.year == other.year)) && ((self.month == other.month)))
                && ((self.day == other.day))) && ((self.hour == other.hour)))
                && ((self.minute == other.minute))) && ((self.second == other.second)))
                && ((self.microsecond == other.microsecond))) && (same_tz))
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2edatetime {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2edatetime_x2edate {
        pub year: i64,
        pub month: i64,
        pub day: i64,
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edate {
        pub fn new(year: i64, month: i64, day: i64) -> Self {
            Self { year, month, day }
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edate {
        pub fn isoformat(&self) -> String {
            let y: String = format!("{}", self.year);
            let mut mo: String = format!("{}", self.month);
            if ((mo.chars().count() as i64) < (2_i64)) {
                mo = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mo).as_str());
                    __sifr_concat
                };
            }
            let mut d: String = format!("{}", self.day);
            if ((d.chars().count() as i64) < (2_i64)) {
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
            ((((self.year == other.year)) && ((self.month == other.month)))
                && ((self.day == other.day)))
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2edate {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    pub fn _is_leap_year(year: i64) -> bool {
        (((year % (4_i64)) == (0_i64)) && ((year % (100_i64)) != (0_i64)))
            || ((year % (400_i64)) == (0_i64))
    }
    pub fn _days_in_year(year: i64) -> i64 {
        if _is_leap_year(year) {
            return 366_i64;
        }
        365_i64
    }
    pub fn _days_in_month(year: i64, month: i64) -> i64 {
        let month_days: Vec<i64> = vec![
            31_i64, 28_i64, 31_i64, 30_i64, 31_i64, 30_i64, 31_i64, 31_i64, 30_i64, 31_i64,
            30_i64, 31_i64
        ];
        let idx: i64 = month - (1_i64);
        let d: Option<i64> = {
            let __sifr_index_list = &month_days;
            let __sifr_index_i = idx;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if (month == (2_i64)) && _is_leap_year(year) {
            return 29_i64;
        }
        if let Some(d) = d {
            return d;
        }
        0_i64
    }
    pub fn _substring(value: &String, start: i64, end: i64) -> String {
        let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let mut result: String = "".to_string();
        let mut i: i64 = start;
        while i < end {
            let ch: Option<String> = __sifr_chars_value
                .get(i as usize)
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                result.push_str((ch).as_str());
            }
            i += 1_i64;
        }
        result
    }
    pub fn _six_digits(value: i64) -> String {
        let mut rendered: String = format!("{}", value);
        let mut __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
        while ((__sifr_chars_rendered.len() as i64) < (6_i64)) {
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
    ) -> Result<(i64, i64, i64, i64, i64, i64), ValueError> {
        let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        if ((__sifr_chars_value.len() as i64) < (19_i64)) {
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
        if ((((({
            let Some(__indexed_char) = __sifr_chars_value
                .get((4_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "-")
            || (({
                let Some(__indexed_char) = __sifr_chars_value
                    .get((7_i64) as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            }) != "-"))
            || (({
                let Some(__indexed_char) = __sifr_chars_value
                    .get((10_i64) as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            }) != "T"))
            || (({
                let Some(__indexed_char) = __sifr_chars_value
                    .get((13_i64) as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            }) != ":"))
            || (({
                let Some(__indexed_char) = __sifr_chars_value
                    .get((16_i64) as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            }) != ":")
        {
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
        let __sifr_try_res: Result<
            Result<(i64, i64, i64, i64, i64, i64), ValueError>,
            ParseError,
        > = (|| {
            let year: i64 = (_substring(value, 0_i64, 4_i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let month: i64 = (_substring(value, 5_i64, 7_i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let day: i64 = (_substring(value, 8_i64, 10_i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let hour: i64 = (_substring(value, 11_i64, 13_i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let minute: i64 = (_substring(value, 14_i64, 16_i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let second: i64 = (_substring(value, 17_i64, 19_i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            return Ok(Ok((year, month, day, hour, minute, second)));
            unreachable!("sifr try/except return capture fell through");
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
    pub fn _timezone_offset_from_text(text: &String) -> Result<i64, ValueError> {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if (text).as_str() == "UTC" {
            return Ok(0_i64);
        }
        if ((__sifr_chars_text.len() as i64) != (9_i64)) {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        if (_substring(text, 0_i64, 3_i64) != "UTC") {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        let sign_value: String = _substring(text, 3_i64, 4_i64);
        if (sign_value != "+") && (sign_value != "-") {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        if (__sifr_chars_text.get((6_i64) as usize).map(|c| c.to_string())
            != Some(":".to_string()))
        {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        let __sifr_try_res: Result<Result<i64, ValueError>, ParseError> = (|| {
            let hours: i64 = (_substring(text, 4_i64, 6_i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let minutes: i64 = (_substring(text, 7_i64, 9_i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let mut offset: i64 = (hours * (3600_i64)) + (minutes * (60_i64));
            if sign_value == "-" {
                offset = -offset;
            }
            return Ok(Ok(offset));
            unreachable!("sifr try/except return capture fell through");
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
            ValueError,
        > = (|| {
            let whole_seconds: i64 = ts as i64;
            let fractional: f64 = ts - (whole_seconds as f64);
            let mut microsecond: i64 = (fractional * (1000000.0_f64)) as i64;
            if microsecond < (0_i64) {
                microsecond = -microsecond;
            }
            let mut adjusted_seconds: i64 = whole_seconds;
            let mut tz_offset_value: i64 = 0_i64;
            let mut tz_has_offset: bool = false;
            if let Some(tz) = tz.as_ref() {
                let tz_text: String = format!("{}", tz);
                let tz_offset: i64 = _timezone_offset_from_text(&tz_text)?;
                adjusted_seconds = whole_seconds + tz_offset;
                tz_offset_value = tz_offset;
                tz_has_offset = true;
            }
            let rendered: String = datetime_from_timestamp(adjusted_seconds as f64)?;
            let parts: (i64, i64, i64, i64, i64, i64) = _parse_datetime_iso(&rendered)?;
            let year_part: Option<i64> = Some((parts).0);
            let month_part: Option<i64> = Some((parts).1);
            let day_part: Option<i64> = Some((parts).2);
            let hour_part: Option<i64> = Some((parts).3);
            let minute_part: Option<i64> = Some((parts).4);
            let second_part: Option<i64> = Some((parts).5);
            let mut year: i64 = 0_i64;
            let mut month: i64 = 1_i64;
            let mut day: i64 = 1_i64;
            let mut hour: i64 = 0_i64;
            let mut minute: i64 = 0_i64;
            let mut second: i64 = 0_i64;
            if let Some(year_part) = year_part {
                year = year_part;
            }
            if let Some(month_part) = month_part {
                month = month_part;
            }
            if let Some(day_part) = day_part {
                day = day_part;
            }
            if let Some(hour_part) = hour_part {
                hour = hour_part;
            }
            if let Some(minute_part) = minute_part {
                minute = minute_part;
            }
            if let Some(second_part) = second_part {
                second = second_part;
            }
            if tz_has_offset {
                return Ok(
                    Ok(
                        __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                            year,
                            month,
                            day,
                            hour,
                            minute,
                            second,
                            microsecond,
                            Some(tz_offset_value),
                        ),
                    ),
                );
            }
            return Ok(
                Ok(
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        year,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                        microsecond,
                        None,
                    ),
                ),
            );
            unreachable!("sifr try/except return capture fell through");
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
    pub fn _from_timestamp_microseconds_with_tz(
        value: i64,
        tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
    ) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
        let whole_seconds: i64 = value / (1000000_i64);
        let microsecond: i64 = value % (1000000_i64);
        let __sifr_try_res: Result<
            Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
            ValueError,
        > = (|| {
            let result: __SifrStdlib_sifr_x2edatetime_x2edatetime = _from_timestamp_with_tz(
                whole_seconds as f64,
                tz,
            )?;
            return Ok(
                Ok(
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        result.year,
                        result.month,
                        result.day,
                        result.hour,
                        result.minute,
                        result.second,
                        microsecond,
                        result._tz_offset,
                    ),
                ),
            );
            unreachable!("sifr try/except return capture fell through");
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
}
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2edate;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2edatetime;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2etimezone;
fn datetime_now() -> String {
    ::sifr_stdlib::time::datetime_now()
}
fn datetime_now_struct() -> Vec<i64> {
    ::sifr_stdlib::time::datetime_now_struct()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
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
fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn time_localtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn _is_leap_year(year: i64) -> bool {
    (((year % (4_i64)) == (0_i64)) && ((year % (100_i64)) != (0_i64)))
        || ((year % (400_i64)) == (0_i64))
}
fn _days_in_year(year: i64) -> i64 {
    if _is_leap_year(year) {
        return 366_i64;
    }
    365_i64
}
fn _days_in_month(year: i64, month: i64) -> i64 {
    let month_days: Vec<i64> = vec![
        31_i64, 28_i64, 31_i64, 30_i64, 31_i64, 30_i64, 31_i64, 31_i64, 30_i64, 31_i64,
        30_i64, 31_i64
    ];
    let idx: i64 = month - (1_i64);
    let d: Option<i64> = {
        let __sifr_index_list = &month_days;
        let __sifr_index_i = idx;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if (month == (2_i64)) && _is_leap_year(year) {
        return 29_i64;
    }
    if let Some(d) = d {
        return d;
    }
    0_i64
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = __sifr_chars_value
            .get(i as usize)
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    result
}
fn _six_digits(value: i64) -> String {
    let mut rendered: String = format!("{}", value);
    let mut __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
    while ((__sifr_chars_rendered.len() as i64) < (6_i64)) {
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
) -> Result<(i64, i64, i64, i64, i64, i64), ValueError> {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    if ((__sifr_chars_value.len() as i64) < (19_i64)) {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    if ((((({
        let Some(__indexed_char) = __sifr_chars_value
            .get((4_i64) as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    }) != "-")
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((7_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "-"))
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((10_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "T"))
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((13_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":"))
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((16_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":")
    {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    let __sifr_try_res: Result<
        Result<(i64, i64, i64, i64, i64, i64), ValueError>,
        ParseError,
    > = (|| {
        let year: i64 = (_substring(value, 0_i64, 4_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let month: i64 = (_substring(value, 5_i64, 7_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let day: i64 = (_substring(value, 8_i64, 10_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let hour: i64 = (_substring(value, 11_i64, 13_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minute: i64 = (_substring(value, 14_i64, 16_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let second: i64 = (_substring(value, 17_i64, 19_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        return Ok(Ok((year, month, day, hour, minute, second)));
        unreachable!("sifr try/except return capture fell through");
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
fn _timezone_offset_from_text(text: &String) -> Result<i64, ValueError> {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (text).as_str() == "UTC" {
        return Ok(0_i64);
    }
    if ((__sifr_chars_text.len() as i64) != (9_i64)) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (_substring(text, 0_i64, 3_i64) != "UTC") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let sign_value: String = _substring(text, 3_i64, 4_i64);
    if (sign_value != "+") && (sign_value != "-") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (__sifr_chars_text.get((6_i64) as usize).map(|c| c.to_string())
        != Some(":".to_string()))
    {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let __sifr_try_res: Result<Result<i64, ValueError>, ParseError> = (|| {
        let hours: i64 = (_substring(text, 4_i64, 6_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minutes: i64 = (_substring(text, 7_i64, 9_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let mut offset: i64 = (hours * (3600_i64)) + (minutes * (60_i64));
        if sign_value == "-" {
            offset = -offset;
        }
        return Ok(Ok(offset));
        unreachable!("sifr try/except return capture fell through");
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
        ValueError,
    > = (|| {
        let whole_seconds: i64 = ts as i64;
        let fractional: f64 = ts - (whole_seconds as f64);
        let mut microsecond: i64 = (fractional * (1000000.0_f64)) as i64;
        if microsecond < (0_i64) {
            microsecond = -microsecond;
        }
        let mut adjusted_seconds: i64 = whole_seconds;
        let mut tz_offset_value: i64 = 0_i64;
        let mut tz_has_offset: bool = false;
        if let Some(tz) = tz.as_ref() {
            let tz_text: String = format!("{}", tz);
            let tz_offset: i64 = _timezone_offset_from_text(&tz_text)?;
            adjusted_seconds = whole_seconds + tz_offset;
            tz_offset_value = tz_offset;
            tz_has_offset = true;
        }
        let rendered: String = datetime_from_timestamp(adjusted_seconds as f64)?;
        let parts: (i64, i64, i64, i64, i64, i64) = _parse_datetime_iso(&rendered)?;
        let year_part: Option<i64> = Some((parts).0);
        let month_part: Option<i64> = Some((parts).1);
        let day_part: Option<i64> = Some((parts).2);
        let hour_part: Option<i64> = Some((parts).3);
        let minute_part: Option<i64> = Some((parts).4);
        let second_part: Option<i64> = Some((parts).5);
        let mut year: i64 = 0_i64;
        let mut month: i64 = 1_i64;
        let mut day: i64 = 1_i64;
        let mut hour: i64 = 0_i64;
        let mut minute: i64 = 0_i64;
        let mut second: i64 = 0_i64;
        if let Some(year_part) = year_part {
            year = year_part;
        }
        if let Some(month_part) = month_part {
            month = month_part;
        }
        if let Some(day_part) = day_part {
            day = day_part;
        }
        if let Some(hour_part) = hour_part {
            hour = hour_part;
        }
        if let Some(minute_part) = minute_part {
            minute = minute_part;
        }
        if let Some(second_part) = second_part {
            second = second_part;
        }
        if tz_has_offset {
            return Ok(
                Ok(
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        year,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                        microsecond,
                        Some(tz_offset_value),
                    ),
                ),
            );
        }
        return Ok(
            Ok(
                __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                    year,
                    month,
                    day,
                    hour,
                    minute,
                    second,
                    microsecond,
                    None,
                ),
            ),
        );
        unreachable!("sifr try/except return capture fell through");
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
fn _from_timestamp_microseconds_with_tz(
    value: i64,
    tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
    let whole_seconds: i64 = value / (1000000_i64);
    let microsecond: i64 = value % (1000000_i64);
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
        ValueError,
    > = (|| {
        let result: __SifrStdlib_sifr_x2edatetime_x2edatetime = _from_timestamp_with_tz(
            whole_seconds as f64,
            tz,
        )?;
        return Ok(
            Ok(
                __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                    result.year,
                    result.month,
                    result.day,
                    result.hour,
                    result.minute,
                    result.second,
                    microsecond,
                    result._tz_offset,
                ),
            ),
        );
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(current);
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            let parts: Vec<i64> = datetime_now_struct();
            let mut yr: i64 = 0_i64;
            let mut mo: i64 = 1_i64;
            let mut dy: i64 = 1_i64;
            let mut hr: i64 = 0_i64;
            let mut mn: i64 = 0_i64;
            let mut sc: i64 = 0_i64;
            for (i, v) in Box::new(
                (parts)
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
            ) {
                if i == (0_i64) {
                    yr = v;
                }
                if i == (1_i64) {
                    mo = v;
                }
                if i == (2_i64) {
                    dy = v;
                }
                if i == (3_i64) {
                    hr = v;
                }
                if i == (4_i64) {
                    mn = v;
                }
                if i == (5_i64) {
                    sc = v;
                }
            }
            if let Some(tz) = tz.as_ref() {
                let __sifr_try_res: Result<
                    __SifrStdlib_sifr_x2edatetime_x2edatetime,
                    ValueError,
                > = (|| {
                    let parsed_offset: i64 = _timezone_offset_from_text(
                        &format!("{}", tz),
                    )?;
                    return Ok(
                        __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                            yr,
                            mo,
                            dy,
                            hr,
                            mn,
                            sc,
                            0_i64,
                            Some(parsed_offset),
                        ),
                    );
                    unreachable!("sifr try/except return capture fell through");
                })();
                match __sifr_try_res {
                    Ok(__sifr_ret_val) => {
                        return __sifr_ret_val;
                    }
                    Err(__sifr_try_err) => {
                        let _e = __sifr_try_err.clone();
                        return __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                            yr,
                            mo,
                            dy,
                            hr,
                            mn,
                            sc,
                            0_i64,
                            None,
                        );
                    }
                }
            }
            return __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                yr,
                mo,
                dy,
                hr,
                mn,
                sc,
                0_i64,
                None,
            );
        }
    }
}
fn today() -> __SifrStdlib_sifr_x2edatetime_x2edate {
    let current: __SifrStdlib_sifr_x2edatetime_x2edatetime = now(&None);
    __SifrStdlib_sifr_x2edatetime_x2edate::new(current.year, current.month, current.day)
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

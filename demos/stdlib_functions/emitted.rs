// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError,
        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError, ValueError,
    };
    pub(super) use ::sifr_runtime::SifrInt;
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn bisect_left<T: Clone + 'static + PartialOrd>(
        a: &[T],
        x: &T,
        lo: SifrInt,
        hi: Option<SifrInt>,
    ) -> SifrInt {
        let mut left: SifrInt = lo;
        if left < SifrInt::from_i64(0) {
            left = SifrInt::from_i64(0);
        }
        let mut right: SifrInt = SifrInt::from(a.len());
        if hi.is_none() {
            right = SifrInt::from(a.len());
        } else if let Some(hi) = hi.clone() {
            if hi < SifrInt::from_i64(0) {
                right = SifrInt::from_i64(0);
            } else if hi > a.len() {
                right = SifrInt::from(a.len());
            } else {
                right = hi;
            }
        }
        while left < right {
            let mid: SifrInt =
                ::std::ops::Add::add(&left, &right).floor_div_known_nonzero(&SifrInt::from_i64(2));
            let val: Option<T> = {
                let sifr_generated_checked_read_collection = &a;
                let sifr_generated_checked_read_index = &mid;
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(val) = val {
                if val < *x {
                    left = ::std::ops::Add::add(&mid, &SifrInt::from_i64(1));
                } else {
                    right = mid;
                }
            } else {
                left = ::std::ops::Add::add(&mid, &SifrInt::from_i64(1));
            }
        }
        left
    }
    pub(super) trait SifrGeneratedAdd: Sized {}
    impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {}
    impl SifrGeneratedAdd for f64 {}
    impl SifrGeneratedAdd for String {}
    pub(super) fn pairwise<T: Clone + 'static>(data: &[T]) -> Vec<Vec<T>> {
        let mut result: Vec<Vec<T>> = Vec::new();
        let mut prev_values: Vec<T> = Vec::new();
        for value in data.iter().cloned() {
            if prev_values.len() > SifrInt::from_i64(0) {
                let mut pair: Vec<T> = Vec::new();
                let prev: Option<T> = {
                    let sifr_generated_checked_read_collection = &prev_values;
                    let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(prev) = prev {
                    pair.push(prev);
                }
                pair.push(value.clone());
                result.push(pair);
                {
                    let sifr_generated_assign_value = value.clone();
                    {
                        let sifr_generated_index_raw = SifrInt::from_i64(0);
                        let sifr_generated_index_normalized =
                            sifr_generated_index_raw.normalize_index_or_len(prev_values.len());
                        if let Some(sifr_generated_elem) =
                            prev_values.get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
            } else {
                prev_values.push(value);
            }
        }
        result
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn batched<T: Clone + 'static>(
        data: &[T],
        n: SifrInt,
    ) -> Result<Vec<Vec<T>>, ValueError> {
        if n <= SifrInt::from_i64(0) {
            return Err(ValueError::new("batched: n must be > 0".to_string()));
        }
        let mut result: Vec<Vec<T>> = Vec::new();
        let mut current_batch: Vec<T> = Vec::new();
        for value in data.iter().cloned() {
            current_batch.push(value);
            if current_batch.len() == n {
                result.push(current_batch.clone());
                current_batch = Vec::new();
            }
        }
        if current_batch.len() > SifrInt::from_i64(0) {
            result.push(current_batch);
        }
        Ok(result)
    }
    pub(super) fn exp(x: f64) -> f64 {
        ::sifr_stdlib::math::exp(x)
    }
    pub(super) const fn isfinite(x: f64) -> bool {
        ::sifr_stdlib::math::isfinite(x)
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn factorial(n: SifrInt) -> SifrInt {
        if n < SifrInt::from_i64(0) {
            return SifrInt::from_i64(0);
        }
        let mut result: SifrInt = SifrInt::from_i64(1);
        let mut i: SifrInt = SifrInt::from_i64(2);
        while i <= n {
            result = ::std::ops::Mul::mul(&result, &i);
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        result
    }
    pub(super) fn gcd(a: SifrInt, b: SifrInt) -> SifrInt {
        let mut x: SifrInt = a;
        let mut y: SifrInt = b;
        if x < SifrInt::from_i64(0) {
            x = ::std::ops::Sub::sub(&SifrInt::from_i64(0), &x);
        }
        if y < SifrInt::from_i64(0) {
            y = ::std::ops::Sub::sub(&SifrInt::from_i64(0), &y);
        }
        while y != SifrInt::from_i64(0) {
            let temp: SifrInt = y.clone();
            y = x.floor_mod_known_nonzero(&y);
            x = temp;
        }
        x
    }
    #[expect(
        clippy::many_single_char_names,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn lcm(a: SifrInt, b: SifrInt) -> SifrInt {
        if a == SifrInt::from_i64(0) {
            return SifrInt::from_i64(0);
        }
        if b == SifrInt::from_i64(0) {
            return SifrInt::from_i64(0);
        }
        let g: SifrInt = gcd(a.clone(), b.clone());
        if g == SifrInt::from_i64(0) {
            return SifrInt::from_i64(0);
        }
        let mut x: SifrInt = a;
        if x < SifrInt::from_i64(0) {
            x = ::std::ops::Sub::sub(&SifrInt::from_i64(0), &x);
        }
        let mut y: SifrInt = b;
        if y < SifrInt::from_i64(0) {
            y = ::std::ops::Sub::sub(&SifrInt::from_i64(0), &y);
        }
        ::std::ops::Mul::mul(&x.floor_div_known_nonzero(&g), &y)
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn comb(n: SifrInt, k: SifrInt) -> SifrInt {
        if k < SifrInt::from_i64(0) {
            return SifrInt::from_i64(0);
        }
        if k > n {
            return SifrInt::from_i64(0);
        }
        if k == SifrInt::from_i64(0) {
            return SifrInt::from_i64(1);
        }
        if k == n {
            return SifrInt::from_i64(1);
        }
        let mut r: SifrInt = k.clone();
        if r > ::std::ops::Sub::sub(&n, &k) {
            r = ::std::ops::Sub::sub(&n, &k);
        }
        let mut result: SifrInt = SifrInt::from_i64(1);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < r {
            result = ::std::ops::Mul::mul(&result, &::std::ops::Sub::sub(&n, &i));
            let divisor: SifrInt = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
            if divisor == SifrInt::from_i64(0) {
                return SifrInt::from_i64(0);
            }
            result = result.floor_div_known_nonzero(&divisor);
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        result
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn perm(n: SifrInt, k: SifrInt) -> SifrInt {
        if k < SifrInt::from_i64(0) {
            return SifrInt::from_i64(0);
        }
        if k > n {
            return SifrInt::from_i64(0);
        }
        let mut result: SifrInt = SifrInt::from_i64(1);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < k {
            result = ::std::ops::Mul::mul(&result, &::std::ops::Sub::sub(&n, &i));
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        result
    }
    pub(super) fn prod(data: &[SifrInt]) -> SifrInt {
        let mut result: SifrInt = SifrInt::from_i64(1);
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for val in data.iter() {
            result = ::std::ops::Mul::mul(&result, val);
        }
        result
    }
    pub(super) fn basename(path: &str) -> String {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = ::std::ops::Sub::sub(
            &SifrInt::from(sifr_generated_chars_path.len()),
            &SifrInt::from_i64(1),
        );
        while i >= SifrInt::from_i64(0) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch
                && ch == "/"
            {
                return {
                    let sifr_generated_slice_src = &sifr_generated_chars_path;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start =
                        ::std::ops::Add::add(&i, &SifrInt::from_i64(1))
                            .clamp_slice_bound(sifr_generated_slice_len);
                    let sifr_generated_slice_stop = sifr_generated_slice_len;
                    String::from_iter(
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .copied(),
                    )
                };
            }
            i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
        }
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(path.len().saturating_add(0usize));
            sifr_generated_concat.push_str(path);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(super) fn stem(path: &str) -> String {
        let base: String = basename(path);
        let sifr_generated_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut i: SifrInt = ::std::ops::Sub::sub(
            &SifrInt::from(sifr_generated_chars_base.len()),
            &SifrInt::from_i64(1),
        );
        while i > SifrInt::from_i64(0) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_base.len());
                sifr_generated_chars_base
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch
                && ch == "."
            {
                return {
                    let sifr_generated_slice_src = &sifr_generated_chars_base;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start = 0;
                    let sifr_generated_slice_stop = i.clamp_slice_bound(sifr_generated_slice_len);
                    String::from_iter(
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .copied(),
                    )
                };
            }
            i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
        }
        base
    }
    pub(super) fn is_absolute(path: &str) -> bool {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        if sifr_generated_chars_path.len() == SifrInt::from_i64(0) {
            return false;
        }
        if sifr_generated_chars_path.len() >= SifrInt::from_i64(3) {
            let colon: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(1);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let sep: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(2);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(colon) = colon
                && let Some(sep) = sep
                && colon == ":"
                && (sep == "/" || sep == "\\")
            {
                return true;
            }
        }
        let first: Option<String> = {
            let sifr_generated_string_index = SifrInt::from_i64(0);
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_path.len());
            sifr_generated_chars_path
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        if let Some(first) = first
            && (first == "/" || first == "\\")
        {
            return true;
        }
        false
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(super) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
            FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
        ),
    }
    impl From<FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatOverflowError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatPrecisionLossError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_float_int(
        value: SifrInt,
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        > = (|| {
            let converted: f64 = value
                .checked_to_f64()
                .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range"
                                    .to_string(),
                            ),
                        )
                    }
                    ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                            FloatPrecisionLossError::new(
                                "exact integer cannot be represented without float precision loss"
                                    .to_string(),
                            ),
                        )
                    }
                })?;
            Ok(Ok(converted))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error;
                    Err(
                        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message,
                        ),
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error;
                    Err(
                        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message,
                        ),
                    )
                }
            })
    }
    pub(super) fn harmonic_mean(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if n == SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "harmonic_mean requires at least one data point".to_string(),
            ));
        }
        let mut total: f64 = 0.0_f64;
        for val in data.iter().copied() {
            if val <= 0.0_f64 {
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    "harmonic_mean requires positive values".to_string(),
                ));
            }
            total += 1.0_f64 / val;
        }
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let numerator: f64 = sifr_generated_float_int(n.clone())?;
            Ok(Ok(numerator / total))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err;
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                error.message,
            ))
        })
    }
    pub(super) fn median_low(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if n == SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "median_low requires at least one data point".to_string(),
            ));
        }
        let sorted_data: Vec<f64> = {
            let mut sifr_generated_sorted_values = data.iter().copied().collect::<Vec<_>>();
            sifr_generated_sorted_values.sort_by(
                |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                    sifr_generated_sorted_left
                        .partial_cmp(sifr_generated_sorted_right)
                        .unwrap_or(::std::cmp::Ordering::Equal)
                },
            );
            sifr_generated_sorted_values
        };
        let mid: SifrInt = n.floor_div_known_nonzero(&SifrInt::from_i64(2));
        if n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == SifrInt::from_i64(0) {
            let val: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index =
                    ::std::ops::Sub::sub(&mid, &SifrInt::from_i64(1));
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .copied()
            };
            let Some(val) = val else {
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    "median_low: index error".to_string(),
                ));
            };
            Ok(val)
        } else {
            let val2: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index = &mid;
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .copied()
            };
            let Some(val2_value_4373ff00edde01ca) = val2 else {
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    "median_low: index error".to_string(),
                ));
            };
            Ok(val2_value_4373ff00edde01ca)
        }
    }
    pub(super) fn median_high(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if n == SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "median_high requires at least one data point".to_string(),
            ));
        }
        let sorted_data: Vec<f64> = {
            let mut sifr_generated_sorted_values = data.iter().copied().collect::<Vec<_>>();
            sifr_generated_sorted_values.sort_by(
                |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                    sifr_generated_sorted_left
                        .partial_cmp(sifr_generated_sorted_right)
                        .unwrap_or(::std::cmp::Ordering::Equal)
                },
            );
            sifr_generated_sorted_values
        };
        let mid: SifrInt = n.floor_div_known_nonzero(&SifrInt::from_i64(2));
        let val: Option<f64> = {
            let sifr_generated_checked_read_collection = &sorted_data;
            let sifr_generated_checked_read_index = &mid;
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .copied()
        };
        let Some(val) = val else {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "median_high: index error".to_string(),
            ));
        };
        Ok(val)
    }
    pub(super) fn capwords(s: &str) -> String {
        let normalized: String = s
            .replace(['\n', '\t'], " ")
            .replace(['\u{b}', '\r'], " ")
            .replace('\u{c}', " ");
        let words: Vec<String> = normalized
            .split(' ')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        let mut result: String = String::new();
        let mut first: bool = true;
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for word in words.iter() {
            if word.chars().count() > SifrInt::from_i64(0) {
                if !first {
                    result.push(' ');
                }
                first = false;
                let cap: String = {
                    let sifr_generated_s = word.clone();
                    let mut sifr_generated_c = sifr_generated_s.chars();
                    sifr_generated_c
                        .next()
                        .map(|f| {
                            f.to_uppercase().to_string() + &sifr_generated_c.as_str().to_lowercase()
                        })
                        .unwrap_or_default()
                };
                result.push_str(cap.as_str());
            }
        }
        result
    }
}
mod sifr_generated_project_nominals {
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("StatisticsError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {}
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatOverflowError {
        pub message: String,
    }
    impl FloatOverflowError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
        #[must_use]
        pub const fn new(message: String) -> Self {
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
use crate::sifr_generated_generated_support::{
    batched, bisect_left, capwords, comb, exp, f64, factorial, gcd, harmonic_mean, is_absolute,
    isfinite, lcm, median_high, median_low, pairwise, perm, prod, stem,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError;
pub use sifr_generated_project_nominals::ValueError;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("=== Math Functions ===");
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(16usize.saturating_add(0usize));
        sifr_generated_concat.push_str("factorial(10) = ");
        sifr_generated_concat.push_str(factorial(SifrInt::from_i64(10)).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("gcd(48, 18) = ");
        sifr_generated_concat.push_str(
            gcd(SifrInt::from_i64(48), SifrInt::from_i64(18))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(12usize.saturating_add(0usize));
        sifr_generated_concat.push_str("lcm(4, 6) = ");
        sifr_generated_concat.push_str(
            lcm(SifrInt::from_i64(4), SifrInt::from_i64(6))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("comb(10, 3) = ");
        sifr_generated_concat.push_str(
            comb(SifrInt::from_i64(10), SifrInt::from_i64(3))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(13usize.saturating_add(0usize));
        sifr_generated_concat.push_str("perm(5, 3) = ");
        sifr_generated_concat.push_str(
            perm(SifrInt::from_i64(5), SifrInt::from_i64(3))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(20usize.saturating_add(0usize));
        sifr_generated_concat.push_str("prod([1,2,3,4,5]) = ");
        sifr_generated_concat.push_str(prod(&nums).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(11usize.saturating_add(0usize));
        sifr_generated_concat.push_str("exp(1.0) = ");
        sifr_generated_concat.push_str(exp(1.0_f64).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(16usize.saturating_add(0usize));
        sifr_generated_concat.push_str("isfinite(1.0) = ");
        sifr_generated_concat.push_str(isfinite(1.0_f64).to_string().as_str());
        sifr_generated_concat
    });
    println!("=== Statistics Functions ===");
    let data: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let even: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64];
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let hm: f64 = harmonic_mean(&data)?;
            let ml: f64 = median_low(&even)?;
            let mh: f64 = median_high(&even)?;
            println!("{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(16usize.saturating_add(0usize));
                sifr_generated_concat.push_str("harmonic_mean = ");
                sifr_generated_concat.push_str(hm.to_string().as_str());
                sifr_generated_concat
            });
            println!("{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(13usize.saturating_add(0usize));
                sifr_generated_concat.push_str("median_low = ");
                sifr_generated_concat.push_str(ml.to_string().as_str());
                sifr_generated_concat
            });
            println!("{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(14usize.saturating_add(0usize));
                sifr_generated_concat.push_str("median_high = ");
                sifr_generated_concat.push_str(mh.to_string().as_str());
                sifr_generated_concat
            });
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let se = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(18usize.saturating_add(0usize));
            sifr_generated_concat.push_str("statistics error: ");
            sifr_generated_concat.push_str(se.message.as_str());
            sifr_generated_concat
        });
    }
    println!("=== String Functions ===");
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(11usize.saturating_add(0usize));
        sifr_generated_concat.push_str("capwords = ");
        sifr_generated_concat.push_str(capwords("hello world test").as_str());
        sifr_generated_concat
    });
    println!("=== Path Functions ===");
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(7usize.saturating_add(0usize));
        sifr_generated_concat.push_str("stem = ");
        sifr_generated_concat.push_str(stem("/docs/report.pdf").as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("is_absolute = ");
        sifr_generated_concat.push_str(is_absolute("/usr/bin").to_string().as_str());
        sifr_generated_concat
    });
    println!("=== Generic Bisect ===");
    let floats: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(27usize.saturating_add(0usize));
        sifr_generated_concat.push_str("bisect_left(floats, 2.5) = ");
        sifr_generated_concat.push_str(
            bisect_left(&floats, &2.5_f64, SifrInt::from_i64(0), None)
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("=== Itertools ===");
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    ];
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(11usize.saturating_add(0usize));
        sifr_generated_concat.push_str("pairwise = ");
        sifr_generated_concat.push_str(format!("{:?}", pairwise(&items)).as_str());
        sifr_generated_concat
    });
    let items2_value_67f5ee13abbe6207: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
        SifrInt::from_i64(6),
        SifrInt::from_i64(7),
        SifrInt::from_i64(8),
    ];
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let bat: Vec<Vec<SifrInt>> =
            batched(&items2_value_67f5ee13abbe6207.clone(), SifrInt::from_i64(3))?;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(10usize.saturating_add(0usize));
            sifr_generated_concat.push_str("batched = ");
            sifr_generated_concat.push_str(format!("{bat:?}").as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error: {}", e.message);
    }
    println!("Done!");
}

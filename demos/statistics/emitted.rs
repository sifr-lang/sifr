// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError,
        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError, ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) use ::std::collections::HashMap;
    pub(crate) fn sqrt(x: f64) -> f64 {
        ::sifr_stdlib::math::sqrt(x)
    }
    pub(crate) fn log(x: f64) -> f64 {
        ::sifr_stdlib::math::log(x)
    }
    pub(crate) fn exp(x: f64) -> f64 {
        ::sifr_stdlib::math::exp(x)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
        ),
    }
    impl From<FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatOverflowError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatPrecisionLossError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(crate) fn sifr_generated_sum(data: &[f64]) -> f64 {
        let mut total: f64 = 0.0_f64;
        for val in data.iter().copied() {
            total += val;
        }
        total
    }
    pub(crate) fn sifr_generated_float_int(
        value: SifrInt,
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        > = (|| {
            let converted: f64 = value
                .clone()
                .checked_to_f64()
                .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range"
                                    .to_string(),
                            ),
                        )
                    }
                    ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
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
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error.clone();
                    Err(
                        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error.clone();
                    Err(
                        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    )
                }
            })
    }
    pub(crate) fn sifr_generated_divide_by_int(
        numerator: f64,
        denominator: SifrInt,
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let divisor: f64 = sifr_generated_float_int(denominator.clone())?;
            Ok(Ok(numerator / divisor))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                error.message.clone(),
            ))
        })
    }
    pub(crate) fn mean(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let count: SifrInt = SifrInt::from(data.len());
        if &count == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ));
        }
        let total: f64 = sifr_generated_sum(data);
        sifr_generated_divide_by_int(total, count.clone())
    }
    pub(crate) fn median(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if &n == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "median requires at least one data point".to_string(),
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
        if &n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0) {
            let a: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index = &mid - &SifrInt::from_i64(1);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let b: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index = mid.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(a) = a
                && let Some(b) = b
            {
                return Ok((a + b) / 2.0_f64);
            }
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "median: index error".to_string(),
            ))
        } else {
            let val: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index = mid.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let Some(val) = val else {
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    "median: index error".to_string(),
                ));
            };
            Ok(val)
        }
    }
    pub(crate) fn variance(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "variance requires at least two data points".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64,),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let avg: f64 = sifr_generated_divide_by_int(sifr_generated_sum(data), n.clone())?;
            Ok((avg,))
        })();
        let (avg,) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut total: f64 = 0.0_f64;
        for val in data.iter().copied() {
            let diff: f64 = val - avg;
            total += diff * diff;
        }
        sifr_generated_divide_by_int(total, &n - &SifrInt::from_i64(1))
    }
    pub(crate) fn stdev(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "stdev requires at least two data points".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64,),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let avg: f64 = sifr_generated_divide_by_int(sifr_generated_sum(data), n.clone())?;
            Ok((avg,))
        })();
        let (avg,) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut total: f64 = 0.0_f64;
        for val in data.iter().copied() {
            let diff: f64 = val - avg;
            total += diff * diff;
        }
        let sifr_generated_try_res: Result<
            (f64,),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let v: f64 = sifr_generated_divide_by_int(total, &n - &SifrInt::from_i64(1))?;
            Ok((v,))
        })();
        let (v,) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        Ok(sqrt(v))
    }
    pub(crate) fn harmonic_mean(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if &n == &SifrInt::from_i64(0) {
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
            let error = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                error.message.clone(),
            ))
        })
    }
    pub(crate) fn geometric_mean(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if &n == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "geometric_mean requires at least one data point".to_string(),
            ));
        }
        let mut log_sum: f64 = 0.0_f64;
        for val in data.iter().copied() {
            if val <= 0.0_f64 {
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    "geometric_mean requires positive values".to_string(),
                ));
            }
            log_sum += log(val);
        }
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let mean_log: f64 = sifr_generated_divide_by_int(log_sum, n.clone())?;
            Ok(Ok(exp(mean_log)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                error.message.clone(),
            ))
        })
    }
    pub(crate) fn mode(
        data: &[SifrInt],
    ) -> Result<SifrInt, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        if &SifrInt::from(data.len()) == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "mode requires at least one data point".to_string(),
            ));
        }
        let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
        for val in data.iter().cloned() {
            let existing: Option<SifrInt> = counts.get(&val).cloned();
            if let Some(existing) = existing.clone() {
                {
                    let sifr_generated_assign_value = &existing + &SifrInt::from_i64(1);
                    {
                        let sifr_generated_assign_key = val.clone();
                        counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            } else {
                let sifr_generated_assign_value = SifrInt::from_i64(1);
                {
                    let sifr_generated_assign_key = val.clone();
                    counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
        let mut best: SifrInt = SifrInt::from_i64(0);
        let mut best_set: bool = false;
        let mut best_count: SifrInt = SifrInt::from_i64(0);
        for val2 in data.iter().cloned() {
            let count2_value_c3423dbe5aaebcf2: Option<SifrInt> = counts.get(&val2).cloned();
            let count2_val: SifrInt = count2_value_c3423dbe5aaebcf2
                .clone()
                .unwrap_or_else(|| SifrInt::from_i64(0));
            if &count2_val > &best_count {
                best_count = count2_val;
                best = val2;
                best_set = true;
            }
        }
        if best_set {
            return Ok(best.clone());
        }
        Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
            "mode: no mode found".to_string(),
        ))
    }
    pub(crate) fn multimode(
        data: &[SifrInt],
    ) -> Result<Vec<SifrInt>, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        if &SifrInt::from(data.len()) == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "multimode requires at least one data point".to_string(),
            ));
        }
        let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
        for val in data.iter().cloned() {
            let existing: Option<SifrInt> = counts.get(&val).cloned();
            if let Some(existing) = existing.clone() {
                {
                    let sifr_generated_assign_value = &existing + &SifrInt::from_i64(1);
                    {
                        let sifr_generated_assign_key = val.clone();
                        counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            } else {
                let sifr_generated_assign_value = SifrInt::from_i64(1);
                {
                    let sifr_generated_assign_key = val.clone();
                    counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
        let mut max_count: SifrInt = SifrInt::from_i64(0);
        for val2 in data.iter().cloned() {
            let count2_value_c3423dbe5aaebcf2: Option<SifrInt> = counts.get(&val2).cloned();
            let count2_val: SifrInt = count2_value_c3423dbe5aaebcf2
                .clone()
                .unwrap_or_else(|| SifrInt::from_i64(0));
            if &count2_val > &max_count {
                max_count = count2_val;
            }
        }
        let mut result: Vec<SifrInt> = Vec::new();
        let mut seen: HashMap<SifrInt, bool> = HashMap::from([]);
        for val3 in data.iter().cloned() {
            let already_opt: Option<bool> = seen.get(&val3).cloned();
            let already: bool = already_opt.is_some_and(|already_opt| already_opt);
            if !already {
                let count3_value_c3423ebe5aaebea5: Option<SifrInt> = counts.get(&val3).cloned();
                let count3_val_value_7442ae8ecb6bc585: SifrInt = count3_value_c3423ebe5aaebea5
                    .clone()
                    .unwrap_or_else(|| SifrInt::from_i64(0));
                if &count3_val_value_7442ae8ecb6bc585 == &max_count {
                    result.push(val3.clone());
                }
                {
                    let sifr_generated_assign_value = true;
                    {
                        let sifr_generated_assign_key = val3.clone();
                        seen.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            }
        }
        Ok(result)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn quantiles(
        data: &[f64],
        n: SifrInt,
    ) -> Result<Vec<f64>, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        if &SifrInt::from(data.len()) < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "quantiles requires at least two data points".to_string(),
            ));
        }
        if &n < &SifrInt::from_i64(1) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "quantiles: n must be at least 1".to_string(),
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
        let m: SifrInt = SifrInt::from(sorted_data.len());
        let mut result: Vec<f64> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(1);
        while &i < &n {
            let sifr_generated_try_res: Result<
                (f64, f64, f64),
                SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
            > = (|| {
                let i_float: f64 = sifr_generated_float_int(i.clone())?;
                let m_float_value_b0fecb9ab83ca525: f64 = sifr_generated_float_int(m.clone())?;
                let n_float_value_15c49f18b6cbd018: f64 = sifr_generated_float_int(n.clone())?;
                Ok((
                    i_float,
                    m_float_value_b0fecb9ab83ca525,
                    n_float_value_15c49f18b6cbd018,
                ))
            })();
            let (i_float, m_float_value_b0fecb9ab83ca525, n_float_value_15c49f18b6cbd018) =
                match sifr_generated_try_res {
                    Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
                    Err(sifr_generated_try_err) => {
                        let error = sifr_generated_try_err.clone();
                        return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message.clone(),
                        ));
                    }
                };
            let idx_f: f64 =
                i_float * m_float_value_b0fecb9ab83ca525 / n_float_value_15c49f18b6cbd018;
            let mut idx: SifrInt = SifrInt::from_i64(0);
            let sifr_generated_try_res: Result<(), ValueError> = (|| {
                let converted_idx: SifrInt =
                    SifrInt::from_f64_trunc(idx_f).ok_or_else(|| ValueError {
                        message: "cannot convert non-finite float to int".to_string(),
                    })?;
                idx = converted_idx;
                Ok(())
            })();
            if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
            let sifr_generated_try_res: Result<
                (f64,),
                SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
            > = (|| {
                let idx_float: f64 = sifr_generated_float_int(idx.clone())?;
                Ok((idx_float,))
            })();
            let (idx_float,) = match sifr_generated_try_res {
                Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
                Err(sifr_generated_try_err) => {
                    let error = sifr_generated_try_err.clone();
                    return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                        error.message.clone(),
                    ));
                }
            };
            let frac: f64 = idx_f - idx_float;
            if &idx >= &m {
                idx = &m - &SifrInt::from_i64(1);
            }
            if &idx < &SifrInt::from_i64(0) {
                idx = SifrInt::from_i64(0);
            }
            let lo: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index = idx.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let mut lo_val: f64 = lo.unwrap_or(0.0_f64);
            if frac > 0.0_f64 {
                let hi_idx: SifrInt = &idx + &SifrInt::from_i64(1);
                if &hi_idx < &m {
                    let hi: Option<f64> = {
                        let sifr_generated_checked_read_collection = &sorted_data;
                        let sifr_generated_checked_read_index = hi_idx.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(hi) = hi {
                        lo_val += frac * (hi - lo_val);
                    }
                }
            }
            result.push(lo_val);
            i = &i + &SifrInt::from_i64(1);
        }
        Ok(result)
    }
    pub(crate) fn covariance(
        x: &[f64],
        y: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(x.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "covariance requires at least two data points".to_string(),
            ));
        }
        if &SifrInt::from(y.len()) != &n {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "covariance: x and y must have the same length".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64, f64),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let mx: f64 = sifr_generated_divide_by_int(sifr_generated_sum(x), n.clone())?;
            let my: f64 = sifr_generated_divide_by_int(sifr_generated_sum(y), n.clone())?;
            Ok((mx, my))
        })();
        let (mx, my) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut total: f64 = 0.0_f64;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            let xi: Option<f64> = {
                let sifr_generated_checked_read_collection = &x;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let yi: Option<f64> = {
                let sifr_generated_checked_read_collection = &y;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(xi) = xi
                && let Some(yi) = yi
            {
                total += (xi - mx) * (yi - my);
            }
            i = &i + &SifrInt::from_i64(1);
        }
        sifr_generated_divide_by_int(total, &n - &SifrInt::from_i64(1))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn correlation(
        x: &[f64],
        y: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(x.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "correlation requires at least two data points".to_string(),
            ));
        }
        if &SifrInt::from(y.len()) != &n {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "correlation: x and y must have the same length".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64, f64),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let mx: f64 = sifr_generated_divide_by_int(sifr_generated_sum(x), n.clone())?;
            let my: f64 = sifr_generated_divide_by_int(sifr_generated_sum(y), n.clone())?;
            Ok((mx, my))
        })();
        let (mx, my) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut cov_num: f64 = 0.0_f64;
        let mut sx_num: f64 = 0.0_f64;
        let mut sy_num_value_0e49c538a785c2b2: f64 = 0.0_f64;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            let xi: Option<f64> = {
                let sifr_generated_checked_read_collection = &x;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let yi: Option<f64> = {
                let sifr_generated_checked_read_collection = &y;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(xi) = xi
                && let Some(yi) = yi
            {
                cov_num += (xi - mx) * (yi - my);
                sx_num += (xi - mx) * (xi - mx);
                sy_num_value_0e49c538a785c2b2 += (yi - my) * (yi - my);
            }
            i = &i + &SifrInt::from_i64(1);
        }
        let sifr_generated_try_res: Result<
            (f64, f64),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let sx_variance: f64 =
                sifr_generated_divide_by_int(sx_num, &n - &SifrInt::from_i64(1))?;
            let sy_variance_value_29a72f81ad7b8e6d: f64 = sifr_generated_divide_by_int(
                sy_num_value_0e49c538a785c2b2,
                &n - &SifrInt::from_i64(1),
            )?;
            let sx: f64 = sqrt(sx_variance);
            let sy: f64 = sqrt(sy_variance_value_29a72f81ad7b8e6d);
            Ok((sx, sy))
        })();
        let (sx, sy) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        if sx == 0.0_f64 {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "correlation: x has zero variance".to_string(),
            ));
        }
        if sy == 0.0_f64 {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "correlation: y has zero variance".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let covariance_value: f64 =
                sifr_generated_divide_by_int(cov_num, &n - &SifrInt::from_i64(1))?;
            Ok(Ok(covariance_value / (sx * sy)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                error.message.clone(),
            ))
        })
    }
    pub(crate) fn linear_regression(
        x: &[f64],
        y: &[f64],
    ) -> Result<Vec<f64>, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(x.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "linear_regression requires at least two data points".to_string(),
            ));
        }
        if &SifrInt::from(y.len()) != &n {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "linear_regression: x and y must have the same length".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64, f64),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let mx: f64 = sifr_generated_divide_by_int(sifr_generated_sum(x), n.clone())?;
            let my: f64 = sifr_generated_divide_by_int(sifr_generated_sum(y), n.clone())?;
            Ok((mx, my))
        })();
        let (mx, my) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut num: f64 = 0.0_f64;
        let mut den: f64 = 0.0_f64;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            let xi: Option<f64> = {
                let sifr_generated_checked_read_collection = &x;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let yi: Option<f64> = {
                let sifr_generated_checked_read_collection = &y;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(xi) = xi
                && let Some(yi) = yi
            {
                num += (xi - mx) * (yi - my);
                den += (xi - mx) * (xi - mx);
            }
            i = &i + &SifrInt::from_i64(1);
        }
        if den == 0.0_f64 {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "linear_regression: x has zero variance".to_string(),
            ));
        }
        let slope: f64 = num / den;
        let intercept: f64 = my - slope * mx;
        let result: Vec<f64> = vec![slope, intercept];
        Ok(result)
    }
    pub(crate) fn assert_vector_eq(actual: &[String], expected: &[String]) {
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
                        .cloned()
                },
                {
                    let sifr_generated_condition_list = &expected;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .cloned()
                }
            );
            i = &i + &SifrInt::from_i64(1);
        }
    }
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
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError;
pub use sifr_generated_project_nominals::ValueError;
fn near(v: f64, target: f64, tol: f64) -> bool {
    if v < target - tol {
        return false;
    }
    if v > target + tol {
        return false;
    }
    true
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn collect_positive_actual() -> Vec<String> {
    let mut actual: Vec<String> = Vec::new();
    let data: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let mut mean_ok: bool = true;
    let mut mean_v_value_2128bd76457bb465: f64 = 0.0_f64;
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_mean: f64 = mean(&data)?;
            mean_v_value_2128bd76457bb465 = out_mean;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        mean_ok = false;
    }
    actual.push((mean_ok && near(mean_v_value_2128bd76457bb465, 3.0_f64, 0.0001_f64)).to_string());
    let mut median_ok: bool = true;
    let mut median_v_value_629d73c6ad2d498a: f64 = 0.0_f64;
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_median_value_e18b9a6a10cf229e: f64 = median(&data)?;
            median_v_value_629d73c6ad2d498a = out_median_value_e18b9a6a10cf229e;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        median_ok = false;
    }
    actual.push(
        (median_ok && near(median_v_value_629d73c6ad2d498a, 3.0_f64, 0.0001_f64)).to_string(),
    );
    let mut variance_ok: bool = true;
    let mut variance_v_value_c6ec119b40af5f5f: f64 = 0.0_f64;
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_variance: f64 = variance(&data)?;
            variance_v_value_c6ec119b40af5f5f = out_variance;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        variance_ok = false;
    }
    actual.push(
        (variance_ok && near(variance_v_value_c6ec119b40af5f5f, 2.5_f64, 0.0001_f64)).to_string(),
    );
    let mut stdev_ok: bool = true;
    let mut stdev_v_value_999a1eeb2e7130ac: f64 = 0.0_f64;
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_stdev: f64 = stdev(&data)?;
            stdev_v_value_999a1eeb2e7130ac = out_stdev;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        stdev_ok = false;
    }
    actual.push(
        (stdev_ok && near(stdev_v_value_999a1eeb2e7130ac, 1.5811_f64, 0.001_f64)).to_string(),
    );
    let mut mode_ok: bool = true;
    let mut mode_v_value_bb35113315d412f3: SifrInt = SifrInt::from_i64(0);
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_mode: SifrInt = mode(&vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
                SifrInt::from_i64(3),
                SifrInt::from_i64(3),
            ])?;
            mode_v_value_bb35113315d412f3 = out_mode;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        mode_ok = false;
    }
    actual.push((mode_ok && &mode_v_value_bb35113315d412f3 == &SifrInt::from_i64(3)).to_string());
    let mut mm_ok: bool = true;
    let mut mm_v: Vec<SifrInt> = Vec::new();
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_mm_value_a8c419f8b8e571ce: Vec<SifrInt> = multimode(&vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
                SifrInt::from_i64(3),
            ])?;
            mm_v = out_mm_value_a8c419f8b8e571ce;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        mm_ok = false;
    }
    actual.push((mm_ok && &SifrInt::from(mm_v.len()) == &SifrInt::from_i64(2)).to_string());
    let mut q_ok: bool = true;
    let mut q_v: Vec<f64> = Vec::new();
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_q: Vec<f64> = quantiles(
                &vec![
                    1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64, 6.0_f64, 7.0_f64, 8.0_f64,
                ],
                SifrInt::from_i64(4),
            )?;
            q_v = out_q;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        q_ok = false;
    }
    actual.push((q_ok && &SifrInt::from(q_v.len()) == &SifrInt::from_i64(3)).to_string());
    let x: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let y: Vec<f64> = vec![2.0_f64, 4.0_f64, 6.0_f64, 8.0_f64, 10.0_f64];
    let mut cov_ok: bool = true;
    let mut cov_v_value_a9f7ab8e40310a86: f64 = 0.0_f64;
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_cov_value_53f72fa1fcbfdd1c: f64 = covariance(&x, &y)?;
            cov_v_value_a9f7ab8e40310a86 = out_cov_value_53f72fa1fcbfdd1c;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        cov_ok = false;
    }
    actual.push((cov_ok && near(cov_v_value_a9f7ab8e40310a86, 5.0_f64, 0.0001_f64)).to_string());
    let mut corr_ok: bool = true;
    let mut corr_v_value_89c9b7db45ca7e3a: f64 = 0.0_f64;
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_corr: f64 = correlation(&x, &y)?;
            corr_v_value_89c9b7db45ca7e3a = out_corr;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        corr_ok = false;
    }
    actual.push((corr_ok && near(corr_v_value_89c9b7db45ca7e3a, 1.0_f64, 0.0001_f64)).to_string());
    let mut lr_ok: bool = true;
    let mut lr_v: Vec<f64> = Vec::new();
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_lr_value_a8c002f8b8e161e2: Vec<f64> = linear_regression(&x, &y)?;
            lr_v = out_lr_value_a8c002f8b8e161e2;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        lr_ok = false;
    }
    let mut lr_slope_ok: bool = false;
    let mut lr_intercept_ok: bool = false;
    if lr_ok {
        let lr_slope: Option<f64> = {
            let sifr_generated_checked_read_collection = &lr_v;
            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let lr_intercept: Option<f64> = {
            let sifr_generated_checked_read_collection = &lr_v;
            let sifr_generated_checked_read_index = SifrInt::from_i64(1);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        if let Some(lr_slope) = lr_slope {
            lr_slope_ok = near(lr_slope, 2.0_f64, 0.0001_f64);
        }
        if let Some(lr_intercept) = lr_intercept {
            lr_intercept_ok = near(lr_intercept, 0.0_f64, 0.0001_f64);
        }
    }
    actual.push(
        (lr_ok
            && &SifrInt::from(lr_v.len()) == &SifrInt::from_i64(2)
            && lr_slope_ok
            && lr_intercept_ok)
            .to_string(),
    );
    let mut hmean_ok_value_d81d368cc0568a61: bool = true;
    let mut hmean_v_value_05026d4b1054e60b: f64 = 0.0_f64;
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_hmean_value_2d8e4460e35f2ab9: f64 =
                harmonic_mean(&vec![2.0_f64, 4.0_f64, 4.0_f64, 8.0_f64])?;
            hmean_v_value_05026d4b1054e60b = out_hmean_value_2d8e4460e35f2ab9;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        hmean_ok_value_d81d368cc0568a61 = false;
    }
    actual.push(
        (hmean_ok_value_d81d368cc0568a61
            && near(
                hmean_v_value_05026d4b1054e60b,
                3.555_555_555_6_f64,
                0.0001_f64,
            ))
        .to_string(),
    );
    let mut gmean_ok_value_92aed8f8945ba566: bool = true;
    let mut gmean_v_value_b72d30944950c71e: f64 = 0.0_f64;
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let out_gmean_value_8f3a392a67552bd4: f64 = geometric_mean(&vec![4.0_f64, 9.0_f64])?;
            gmean_v_value_b72d30944950c71e = out_gmean_value_8f3a392a67552bd4;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        gmean_ok_value_92aed8f8945ba566 = false;
    }
    actual.push(
        (gmean_ok_value_92aed8f8945ba566
            && near(gmean_v_value_b72d30944950c71e, 6.0_f64, 0.0001_f64))
        .to_string(),
    );
    actual
}
fn collect_error_actual_ok() -> Vec<bool> {
    let mut actual_ok: Vec<bool> = Vec::new();
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let bad_mean: f64 = mean(&Vec::new())?;
            let _ = bad_mean.to_string();
            actual_ok.push(true);
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        actual_ok.push(false);
    }
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let bad_hmean_value_50f3033417df77fa: f64 = harmonic_mean(&vec![0.0_f64, 1.0_f64])?;
            let _ = bad_hmean_value_50f3033417df77fa.to_string();
            actual_ok.push(true);
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        actual_ok.push(false);
    }
    actual_ok
}
fn main() {
    let expected: Vec<String> = vec![
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
    ];
    let actual: Vec<String> = collect_positive_actual();
    assert_vector_eq(&actual, &expected);
    let expected_ok: Vec<bool> = vec![false, false];
    let actual_ok: Vec<bool> = collect_error_actual_ok();
    assert_bool_vector_eq(&actual_ok, &expected_ok);
    println!("statistics parity demo: pass");
}

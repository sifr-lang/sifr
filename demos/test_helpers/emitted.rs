// src/main.rs
mod sifr_generated_project_nominals {}
use ::sifr_runtime::SifrInt;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
{
    SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
    SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(FloatPrecisionLossError),
}
impl From<FloatOverflowError>
for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
    fn from(value: FloatOverflowError) -> Self {
        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
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
#[derive(Clone, PartialEq, Eq, Hash)]
struct SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
    message: String,
}
impl SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
    const fn new(message: String) -> Self {
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
fn sifr_generated_sum(data: &[f64]) -> f64 {
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        total += val;
    }
    total
}
fn sifr_generated_float_int(
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
                            "exact integer is outside the finite float range".to_string(),
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
fn sifr_generated_divide_by_int(
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
fn mean(data: &[f64]) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
    let count: SifrInt = SifrInt::from(data.len());
    if &count == &SifrInt::from_i64(0) {
        return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
            "mean requires at least one data point".to_string(),
        ));
    }
    let total: f64 = sifr_generated_sum(data);
    sifr_generated_divide_by_int(total, count.clone())
}
fn variance(data: &[f64]) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
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
fn pvariance(data: &[f64]) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if &n == &SifrInt::from_i64(0) {
        return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
            "pvariance requires at least one data point".to_string(),
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
    sifr_generated_divide_by_int(total, n.clone())
}
fn stdev(data: &[f64]) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
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
fn pstdev(data: &[f64]) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if &n == &SifrInt::from_i64(0) {
        return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
            "pstdev requires at least one data point".to_string(),
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
        let v: f64 = sifr_generated_divide_by_int(total, n.clone())?;
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FloatOverflowError {
    message: String,
}
impl FloatOverflowError {
    const fn new(message: String) -> Self {
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
struct FloatPrecisionLossError {
    message: String,
}
impl FloatPrecisionLossError {
    const fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for FloatPrecisionLossError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for FloatPrecisionLossError {}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let data: Vec<f64> = vec![
        2.0_f64, 4.0_f64, 4.0_f64, 4.0_f64, 5.0_f64, 5.0_f64, 7.0_f64, 9.0_f64,
    ];
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let m: f64 = mean(&data)?;
            let sv: f64 = variance(&data)?;
            let pv: f64 = pvariance(&data)?;
            let sd: f64 = stdev(&data)?;
            let pd: f64 = pstdev(&data)?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(7usize);
                sifr_generated_concat.push_str("mean = ");
                sifr_generated_concat.push_str(m.to_string().as_str());
                sifr_generated_concat
            });
            assert_eq!(format!("mean = {m}"), "mean = 5");
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(18usize);
                sifr_generated_concat.push_str("sample variance = ");
                sifr_generated_concat.push_str(sv.to_string().as_str());
                sifr_generated_concat
            });
            assert_eq!(
                format!("sample variance = {sv}"),
                "sample variance = 4.571428571428571"
            );
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(22usize);
                sifr_generated_concat.push_str("population variance = ");
                sifr_generated_concat.push_str(pv.to_string().as_str());
                sifr_generated_concat
            });
            assert_eq!(
                format!("population variance = {pv}"),
                "population variance = 4"
            );
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(15usize);
                sifr_generated_concat.push_str("sample stdev = ");
                sifr_generated_concat.push_str(sd.to_string().as_str());
                sifr_generated_concat
            });
            assert_eq!(
                format!("sample stdev = {sd}"),
                "sample stdev = 2.138089935299395"
            );
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(19usize);
                sifr_generated_concat.push_str("population stdev = ");
                sifr_generated_concat.push_str(pd.to_string().as_str());
                sifr_generated_concat
            });
            assert_eq!(format!("population stdev = {pd}"), "population stdev = 2");
            {
                let sifr_generated_lhs = m;
                let sifr_generated_rhs = 5.0_f64;
                let sifr_generated_tol = 0.001_f64;
                assert!(
                    sifr_generated_lhs == sifr_generated_rhs
                        || (sifr_generated_lhs - sifr_generated_rhs).abs() <= sifr_generated_tol,
                    "assert_almost_eq failed: {} != {} (tolerance {})",
                    sifr_generated_lhs,
                    sifr_generated_rhs,
                    sifr_generated_tol
                )
            };
            {
                let sifr_generated_lhs = sv;
                let sifr_generated_rhs = 4.571_f64;
                let sifr_generated_tol = 0.01_f64;
                assert!(
                    sifr_generated_lhs == sifr_generated_rhs
                        || (sifr_generated_lhs - sifr_generated_rhs).abs() <= sifr_generated_tol,
                    "assert_almost_eq failed: {} != {} (tolerance {})",
                    sifr_generated_lhs,
                    sifr_generated_rhs,
                    sifr_generated_tol
                )
            };
            {
                let sifr_generated_lhs = pv;
                let sifr_generated_rhs = 4.0_f64;
                let sifr_generated_tol = 0.001_f64;
                assert!(
                    sifr_generated_lhs == sifr_generated_rhs
                        || (sifr_generated_lhs - sifr_generated_rhs).abs() <= sifr_generated_tol,
                    "assert_almost_eq failed: {} != {} (tolerance {})",
                    sifr_generated_lhs,
                    sifr_generated_rhs,
                    sifr_generated_tol
                )
            };
            {
                let sifr_generated_lhs = sd;
                let sifr_generated_rhs = 2.138_f64;
                let sifr_generated_tol = 0.01_f64;
                assert!(
                    sifr_generated_lhs == sifr_generated_rhs
                        || (sifr_generated_lhs - sifr_generated_rhs).abs() <= sifr_generated_tol,
                    "assert_almost_eq failed: {} != {} (tolerance {})",
                    sifr_generated_lhs,
                    sifr_generated_rhs,
                    sifr_generated_tol
                )
            };
            {
                let sifr_generated_lhs = pd;
                let sifr_generated_rhs = 2.0_f64;
                let sifr_generated_tol = 0.001_f64;
                assert!(
                    sifr_generated_lhs == sifr_generated_rhs
                        || (sifr_generated_lhs - sifr_generated_rhs).abs() <= sifr_generated_tol,
                    "assert_almost_eq failed: {} != {} (tolerance {})",
                    sifr_generated_lhs,
                    sifr_generated_rhs,
                    sifr_generated_tol
                )
            };
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(18usize);
            sifr_generated_concat.push_str("statistics error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
        assert_eq!(
            format!("statistics error: {}", e.message.clone()),
            "All assertions passed!"
        );
    }
    assert!(
        &SifrInt::from_i64(10) > &SifrInt::from_i64(5),
        "assert_gt failed: {} is not > {}",
        SifrInt::from_i64(10),
        SifrInt::from_i64(5)
    );
    assert!(
        &SifrInt::from_i64(3) < &SifrInt::from_i64(7),
        "assert_lt failed: {} is not < {}",
        SifrInt::from_i64(3),
        SifrInt::from_i64(7)
    );
    assert!(
        &SifrInt::from_i64(100) > &SifrInt::from_i64(0),
        "assert_gt failed: {} is not > {}",
        SifrInt::from_i64(100),
        SifrInt::from_i64(0)
    );
    assert!(
        &SifrInt::from_i64(0) < &SifrInt::from_i64(1),
        "assert_lt failed: {} is not < {}",
        SifrInt::from_i64(0),
        SifrInt::from_i64(1)
    );
    println!("All assertions passed!");
}

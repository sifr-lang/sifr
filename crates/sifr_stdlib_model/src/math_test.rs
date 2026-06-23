use super::IntrinsicModule;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

pub(super) fn intrinsic_math() -> IntrinsicModule {
    let mut functions = HashMap::new();
    let mut constants = HashMap::new();

    // sqrt(x: float) -> float
    functions.insert(
        "sqrt".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // floor(x: float) -> int
    functions.insert(
        "floor".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Int),
    );

    // ceil(x: float) -> int
    functions.insert(
        "ceil".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Int),
    );

    // abs_val(x: float) -> float
    functions.insert(
        "abs_val".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // log(x: float) -> float
    functions.insert(
        "log".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // cbrt(x: float) -> float
    functions.insert(
        "cbrt".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // exp2(x: float) -> float
    functions.insert(
        "exp2".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // sin(x: float) -> float
    functions.insert(
        "sin".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // cos(x: float) -> float
    functions.insert(
        "cos".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // tan(x: float) -> float
    functions.insert(
        "tan".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // pow_val(x: float, y: float) -> float
    functions.insert(
        "pow_val".to_string(),
        FunctionType::all_borrow(
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // min_val(a: float, b: float) -> float
    functions.insert(
        "min_val".to_string(),
        FunctionType::all_borrow(
            vec![
                ("a".to_string(), Type::Float),
                ("b".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // max_val(a: float, b: float) -> float
    functions.insert(
        "max_val".to_string(),
        FunctionType::all_borrow(
            vec![
                ("a".to_string(), Type::Float),
                ("b".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // round_val(x: float) -> int
    functions.insert(
        "round_val".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Int),
    );

    // asin(x: float) -> float
    functions.insert(
        "asin".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // acos(x: float) -> float
    functions.insert(
        "acos".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // atan(x: float) -> float
    functions.insert(
        "atan".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // atan2(y: float, x: float) -> float
    functions.insert(
        "atan2".to_string(),
        FunctionType::all_borrow(
            vec![
                ("y".to_string(), Type::Float),
                ("x".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // sinh(x: float) -> float
    functions.insert(
        "sinh".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // cosh(x: float) -> float
    functions.insert(
        "cosh".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // tanh(x: float) -> float
    functions.insert(
        "tanh".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // log10(x: float) -> float
    functions.insert(
        "log10".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // log2(x: float) -> float
    functions.insert(
        "log2".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // degrees(x: float) -> float (radians to degrees)
    functions.insert(
        "degrees".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // radians(x: float) -> float (degrees to radians)
    functions.insert(
        "radians".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // isnan(x: float) -> bool
    functions.insert(
        "isnan".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Bool),
    );

    // isinf(x: float) -> bool
    functions.insert(
        "isinf".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Bool),
    );

    // trunc(x: float) -> int
    functions.insert(
        "trunc".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Int),
    );

    // copysign(x: float, y: float) -> float
    functions.insert(
        "copysign".to_string(),
        FunctionType::all_borrow(
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // signbit(x: float) -> bool
    functions.insert(
        "signbit".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Bool),
    );

    // fmod(x: float, y: float) -> float
    functions.insert(
        "fmod".to_string(),
        FunctionType::all_borrow(
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // remainder(x: float, y: float) -> float
    functions.insert(
        "remainder".to_string(),
        FunctionType::all_borrow(
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // hypot(x: float, y: float) -> float
    functions.insert(
        "hypot".to_string(),
        FunctionType::all_borrow(
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // fma(x: float, y: float, z: float) -> float
    functions.insert(
        "fma".to_string(),
        FunctionType::all_borrow(
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
                ("z".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // fmax(x: float, y: float) -> float
    functions.insert(
        "fmax".to_string(),
        FunctionType::all_borrow(
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // fmin(x: float, y: float) -> float
    functions.insert(
        "fmin".to_string(),
        FunctionType::all_borrow(
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // exp(x: float) -> float
    functions.insert(
        "exp".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // expm1(x: float) -> float
    functions.insert(
        "expm1".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // log1p(x: float) -> float
    functions.insert(
        "log1p".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // fabs(x: float) -> float
    functions.insert(
        "fabs".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // isfinite(x: float) -> bool
    functions.insert(
        "isfinite".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Bool),
    );

    // isnormal(x: float) -> bool
    functions.insert(
        "isnormal".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Bool),
    );

    // issubnormal(x: float) -> bool
    functions.insert(
        "issubnormal".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Bool),
    );

    // acosh(x: float) -> float
    functions.insert(
        "acosh".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // asinh(x: float) -> float
    functions.insert(
        "asinh".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // atanh(x: float) -> float
    functions.insert(
        "atanh".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // isqrt(n: int) -> int
    functions.insert(
        "isqrt".to_string(),
        FunctionType::all_borrow(vec![("n".to_string(), Type::Int)], Type::Int),
    );

    // dist(p: list[float], q: list[float]) -> float
    functions.insert(
        "dist".to_string(),
        FunctionType::all_borrow(
            vec![
                ("p".to_string(), Type::List(Box::new(Type::Float))),
                ("q".to_string(), Type::List(Box::new(Type::Float))),
            ],
            Type::Float,
        ),
    );

    // fsum(data: list[float]) -> float
    functions.insert(
        "fsum".to_string(),
        FunctionType::all_borrow(
            vec![("data".to_string(), Type::List(Box::new(Type::Float)))],
            Type::Float,
        ),
    );

    // sumprod(p: list[float], q: list[float]) -> float
    functions.insert(
        "sumprod".to_string(),
        FunctionType::all_borrow(
            vec![
                ("p".to_string(), Type::List(Box::new(Type::Float))),
                ("q".to_string(), Type::List(Box::new(Type::Float))),
            ],
            Type::Float,
        ),
    );

    // erf(x: float) -> float
    functions.insert(
        "erf".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // erfc(x: float) -> float
    functions.insert(
        "erfc".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // gamma(x: float) -> float
    functions.insert(
        "gamma".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // lgamma(x: float) -> float
    functions.insert(
        "lgamma".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // frexp(x: float) -> list[float] ([mantissa, exponent_as_float])
    functions.insert(
        "frexp".to_string(),
        FunctionType::all_borrow(
            vec![("x".to_string(), Type::Float)],
            Type::List(Box::new(Type::Float)),
        ),
    );

    // ldexp(m: float, e: int) -> float
    functions.insert(
        "ldexp".to_string(),
        FunctionType::all_borrow(
            vec![("m".to_string(), Type::Float), ("e".to_string(), Type::Int)],
            Type::Float,
        ),
    );

    // modf(x: float) -> list[float] ([fractional_part, integer_part])
    functions.insert(
        "modf".to_string(),
        FunctionType::all_borrow(
            vec![("x".to_string(), Type::Float)],
            Type::List(Box::new(Type::Float)),
        ),
    );

    // nextafter(x: float, y: float) -> float
    functions.insert(
        "nextafter".to_string(),
        FunctionType::all_borrow(
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // ulp(x: float) -> float
    functions.insert(
        "ulp".to_string(),
        FunctionType::all_borrow(vec![("x".to_string(), Type::Float)], Type::Float),
    );

    // Constants
    constants.insert("pi".to_string(), Type::Float);
    constants.insert("e".to_string(), Type::Float);
    constants.insert("tau".to_string(), Type::Float);
    constants.insert("inf".to_string(), Type::Float);
    constants.insert("nan".to_string(), Type::Float);

    IntrinsicModule {
        functions,
        constants,
    }
}

/// _sifr.test — Test assertion intrinsics
pub(super) fn intrinsic_test() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // assert_eq(actual: Any, expected: Any) -> None
    functions.insert(
        "assert_eq".to_string(),
        FunctionType::all_borrow(
            vec![
                ("actual".to_string(), Type::Any),
                ("expected".to_string(), Type::Any),
            ],
            Type::None,
        ),
    );

    // assert_ne(actual: Any, expected: Any) -> None
    functions.insert(
        "assert_ne".to_string(),
        FunctionType::all_borrow(
            vec![
                ("actual".to_string(), Type::Any),
                ("expected".to_string(), Type::Any),
            ],
            Type::None,
        ),
    );

    // assert_true(value: bool) -> None
    functions.insert(
        "assert_true".to_string(),
        FunctionType::all_borrow(vec![("value".to_string(), Type::Bool)], Type::None),
    );

    // assert_false(value: bool) -> None
    functions.insert(
        "assert_false".to_string(),
        FunctionType::all_borrow(vec![("value".to_string(), Type::Bool)], Type::None),
    );

    // assert_almost_eq(actual: float, expected: float, tolerance: float) -> None
    functions.insert(
        "assert_almost_eq".to_string(),
        FunctionType::all_borrow(
            vec![
                ("actual".to_string(), Type::Float),
                ("expected".to_string(), Type::Float),
                ("tolerance".to_string(), Type::Float),
            ],
            Type::None,
        ),
    );

    // assert_gt(a: int, b: int) -> None
    functions.insert(
        "assert_gt".to_string(),
        FunctionType::all_borrow(
            vec![("a".to_string(), Type::Int), ("b".to_string(), Type::Int)],
            Type::None,
        ),
    );

    // assert_lt(a: int, b: int) -> None
    functions.insert(
        "assert_lt".to_string(),
        FunctionType::all_borrow(
            vec![("a".to_string(), Type::Int), ("b".to_string(), Type::Int)],
            Type::None,
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

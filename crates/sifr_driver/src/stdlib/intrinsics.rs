pub(super) fn intrinsic_constant_rust_expr(module: &str, name: &str) -> Option<&'static str> {
    match (module, name) {
        ("_sifr.math", "pi") => Some("std::f64::consts::PI"),
        ("_sifr.math", "e") => Some("std::f64::consts::E"),
        ("_sifr.math", "tau") => Some("std::f64::consts::TAU"),
        ("_sifr.math", "inf") => Some("f64::INFINITY"),
        ("_sifr.math", "nan") => Some("f64::NAN"),
        _ => None,
    }
}
